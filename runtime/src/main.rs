use clap_lex::RawArgs;
use rquickjs::{CatchResultExt, CaughtError, Context, Module, Runtime};
use std::error::Error;
use std::fs;

mod module_builder;

const SECTION_NAME: &str = "mdeno_js";

fn main() -> Result<(), Box<dyn Error>> {
    // Check if this executable has embedded JavaScript code and no arguments
    if std::env::args().len() == 1 {
        if let Ok(embedded_code) = extract_embedded_js() {
            return run_js_code(&embedded_code);
        }
    }

    let raw = RawArgs::from_args();
    let mut cursor = raw.cursor();
    raw.next(&mut cursor); // skip program name

    let mut file_path: Option<String> = None;
    let mut is_compile = false;

    if let Some(arg) = raw.next(&mut cursor) {
        if let Ok(value) = arg.to_value() {
            match value {
                "compile" => {
                    is_compile = true;
                    if let Some(file_arg) = raw.next(&mut cursor) {
                        if let Ok(file_value) = file_arg.to_value() {
                            file_path = Some(file_value.to_string());
                        }
                    }
                }
                "run" => {
                    if let Some(file_arg) = raw.next(&mut cursor) {
                        if let Ok(file_value) = file_arg.to_value() {
                            file_path = Some(file_value.to_string());
                        }
                    }
                }
                _ if !value.starts_with('-') => {
                    // No subcommand, treat as file path (run mode)
                    file_path = Some(value.to_string());
                }
                _ => {}
            }
        }
    }

    let file_path = file_path.ok_or("JavaScript file is required")?;

    // Convert file path to absolute path
    let file_path_buf = std::path::Path::new(&file_path);
    let absolute_file_path = if file_path_buf.is_absolute() {
        file_path_buf.to_path_buf()
    } else {
        std::env::current_dir()?.join(file_path_buf)
    };

    // Convert to native path separator
    let absolute_file_path_str = absolute_file_path
        .components()
        .collect::<std::path::PathBuf>()
        .display()
        .to_string();

    if is_compile {
        let output_name = absolute_file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");

        compile_js_to_executable(&absolute_file_path_str, output_name)?;
        println!("Compiled {} to {}", file_path, output_name);
    } else {
        let js_code = fs::read_to_string(&absolute_file_path)?;
        run_js_code_with_path(&js_code, &absolute_file_path_str)?;
    }

    Ok(())
}

fn run_js_code(js_code: &str) -> Result<(), Box<dyn Error>> {
    run_js_code_with_path(js_code, "")
}

fn run_js_code_with_path(js_code: &str, script_path: &str) -> Result<(), Box<dyn Error>> {
    use module_builder::ModuleBuilder;
    use std::sync::Arc;

    smol::block_on(async {
        let runtime = Runtime::new()?;

        // Build module configuration
        let (_global_attachment, module_registry) = ModuleBuilder::default().build();
        let registry = Arc::new(module_registry);

        // Set module loader before creating context
        runtime.set_loader(
            module_builder::NodeResolver::new(registry.clone()),
            module_builder::NodeLoader::new(registry.clone()),
        );

        let context = Context::full(&runtime)?;

        context.with(|ctx| -> Result<(), Box<dyn Error>> {
            setup_extensions(&ctx, script_path)?;

            let effective_path = if script_path.is_empty() {
                "./$mdeno$eval.js"
            } else {
                script_path
            };

            let result = {
                Module::evaluate(ctx.clone(), effective_path, js_code)
                    .and_then(|m| m.finish::<()>())
            };

            if let Err(caught) = result.catch(&ctx) {
                match caught {
                    CaughtError::Exception(exception) => {
                        if let Some(message) = exception.message() {
                            eprintln!("Error: {}", message);
                        }
                        if let Some(stack) = exception.stack() {
                            eprintln!("{}", stack);
                        }
                    }
                    CaughtError::Value(value) => {
                        eprintln!("Error: {:?}", value);
                    }
                    CaughtError::Error(error) => {
                        eprintln!("Error: {:?}", error);
                    }
                }
                std::process::exit(1);
            }

            // Execute all pending jobs (promises, microtasks)
            while ctx.execute_pending_job() {}

            Ok(())
        })?;

        Ok(())
    })
}

fn compile_js_to_executable(js_file: &str, output_name: &str) -> Result<(), Box<dyn Error>> {
    let js_code = fs::read_to_string(js_file)?;

    // Get current executable path
    let current_exe = std::env::current_exe()?;
    let exe_bytes = fs::read(&current_exe)?;

    // Output executable name
    let output_exe = if cfg!(windows) {
        format!("{}.exe", output_name)
    } else {
        output_name.to_string()
    };

    // Use libsui to embed JavaScript code
    let mut output_file = fs::File::create(&output_exe)?;

    #[cfg(target_os = "windows")]
    {
        use libsui::PortableExecutable;
        PortableExecutable::from(&exe_bytes)?
            .write_resource(SECTION_NAME, js_code.as_bytes().to_vec())?
            .build(&mut output_file)?;
    }

    #[cfg(target_os = "macos")]
    {
        use libsui::Macho;
        Macho::from(exe_bytes)?
            .write_section(SECTION_NAME, js_code.as_bytes().to_vec())?
            .build(&mut output_file)?;
    }

    #[cfg(target_os = "linux")]
    {
        use libsui::Elf;
        let elf = Elf::new(&exe_bytes);
        elf.append(SECTION_NAME, js_code.as_bytes(), &mut output_file)?;
    }

    let file_size = fs::metadata(&output_exe)?.len();
    let size_mb = file_size as f64 / 1024.0 / 1024.0;

    println!("Successfully created: {}", output_exe);
    println!("Size: {:.2} MB", size_mb);

    Ok(())
}

fn extract_embedded_js() -> Result<String, Box<dyn Error>> {
    let data = libsui::find_section(SECTION_NAME)?.ok_or("No embedded JavaScript found")?;

    let js_code = String::from_utf8(data.to_vec())
        .map_err(|e| format!("Invalid UTF-8 in embedded JS: {}", e))?;

    Ok(js_code)
}

fn setup_extensions(ctx: &rquickjs::Ctx, _script_path: &str) -> Result<(), Box<dyn Error>> {
    use module_builder::ModuleBuilder;

    // Initialize mdeno.internal object
    ctx.eval::<(), _>(
        r#"if (!globalThis[Symbol.for("mdeno.internal")]) {
  globalThis[Symbol.for("mdeno.internal")] = {};
}"#,
    )?;

    // Build module configuration using default (feature-based)
    let builder = ModuleBuilder::default();
    let (global_attachment, _module_registry) = builder.build();
    global_attachment.attach(ctx)?;

    Ok(())
}
