use clap::{Arg, Command};
use quickjs_rusty::{Context, ExecutionError};
use std::fs;
use std::io::{Read, Write};

mod ext;
mod internal;
mod node;

const MAGIC_MARKER: &[u8] = b"__JS_CODE_START__";
const MAGIC_END: &[u8] = b"__JS_CODE_END__";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("mnode")
        .about("Minimal JavaScript runtime for CLI tool")
        .arg(Arg::new("file").help("JavaScript file to run").index(1))
        .arg(
            Arg::new("compile")
                .short('c')
                .long("compile")
                .help("Compile JavaScript into a self-contained executable")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    // Check if this executable has embedded JavaScript code and no arguments
    if std::env::args().len() == 1 {
        if let Ok(embedded_code) = extract_embedded_js() {
            return run_js_code(&embedded_code);
        }
    }

    let file_path = matches
        .get_one::<String>("file")
        .ok_or("JavaScript file is required")?;

    // Convert file path to absolute path
    let file_path_buf = std::path::Path::new(file_path);
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

    let is_compile = matches.get_flag("compile");

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

fn run_js_code(js_code: &str) -> Result<(), Box<dyn std::error::Error>> {
    run_js_code_with_path(js_code, "")
}

fn run_js_code_with_path(
    js_code: &str,
    script_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let context = Context::builder().build()?;

    context.add_callback("__print", |msg: String| -> i32 {
        println!("{}", msg);
        0
    })?;

    setup_extensions(&context, script_path)?;

    let result = if js_code.contains("import ") || js_code.contains("export ") {
        context.eval_module(js_code, true)
    } else {
        context.eval(js_code, true)
    };

    if let Err(e) = result {
        match e {
            ExecutionError::Exception(js_value) => {
                let error_message = js_value
                    .to_string()
                    .unwrap_or_else(|_| "Unknown error".to_string());
                eprintln!("{}", error_message);
            }
            _ => {
                eprintln!("{:?}", e);
            }
        }
        std::process::exit(1);
    }

    Ok(())
}

fn compile_js_to_executable(
    js_file: &str,
    output_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let js_code = fs::read_to_string(js_file)?;

    // Get current executable path
    let current_exe = std::env::current_exe()?;

    // Output executable name
    let output_exe = if cfg!(windows) {
        format!("{}.exe", output_name)
    } else {
        output_name.to_string()
    };

    // Copy current executable
    fs::copy(&current_exe, &output_exe)?;

    // Append JavaScript code with markers
    let mut output_file = fs::OpenOptions::new().append(true).open(&output_exe)?;

    output_file.write_all(MAGIC_MARKER)?;
    output_file.write_all(js_code.as_bytes())?;
    output_file.write_all(MAGIC_END)?;

    let file_size = fs::metadata(&output_exe)?.len();
    let size_mb = file_size as f64 / 1024.0 / 1024.0;

    println!("Successfully created: {}", output_exe);
    println!("Size: {:.1} MB", size_mb);

    Ok(())
}

fn extract_embedded_js() -> Result<String, Box<dyn std::error::Error>> {
    let exe_path = std::env::current_exe()?;
    let mut file = fs::File::open(&exe_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    if let Some(start_pos) = find_pattern(&buffer, MAGIC_MARKER) {
        let code_start = start_pos + MAGIC_MARKER.len();
        if let Some(end_pos) = find_pattern(&buffer[code_start..], MAGIC_END) {
            let js_code = &buffer[code_start..code_start + end_pos];
            return Ok(String::from_utf8_lossy(js_code).to_string());
        }
    }

    Err("No embedded JavaScript found".into())
}

fn find_pattern(data: &[u8], pattern: &[u8]) -> Option<usize> {
    data.windows(pattern.len())
        .rposition(|window| window == pattern)
}

fn setup_extensions(
    context: &Context,
    script_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    context.eval(internal::load_setup(), false)?;

    // Navigator
    ext::navigator::setup(context)?;
    context.eval(&ext::load_navigator(), false)?;

    // URL
    ext::url::setup(context)?;
    context.eval(&ext::load_url(), false)?;

    // Load JS modules
    context.eval(&ext::load_console(), false)?;

    // Node.js modules
    node::fs::setup(context)?;
    node::process::setup(context, script_path)?;
    node::set_module_loader(context)?;

    // Global process
    context.eval("globalThis.process = { env: JSON.parse(globalThis[Symbol.for('mnode.internal')].getEnv()), argv: JSON.parse(globalThis[Symbol.for('mnode.internal')].getArgv()), exit: (code = 0) => globalThis[Symbol.for('mnode.internal')].exit(code) };", false)?;

    Ok(())
}
