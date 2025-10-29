use mdeno_utils::{ModuleDef, add_internal_function};
use rquickjs::Ctx;
use std::error::Error;
use std::path::Path;

pub fn init(ctx: &Ctx<'_>) -> rquickjs::Result<()> {
    setup_internal(ctx).map_err(|_| rquickjs::Error::Unknown)?;
    Ok(())
}

pub struct ProcessModule;

impl ModuleDef for ProcessModule {
    fn init(ctx: &Ctx<'_>) -> rquickjs::Result<()> {
        setup_internal(ctx).map_err(|_| rquickjs::Error::Unknown)?;
        Ok(())
    }

    fn name() -> &'static str {
        "node:process"
    }

    fn source() -> &'static str {
        include_str!("process.js")
    }
}

fn setup_internal(ctx: &Ctx) -> Result<(), Box<dyn Error>> {
    add_internal_function!(ctx, "getEnv", || get_env().unwrap_or_else(|e| e));
    add_internal_function!(ctx, "getArgv", || get_argv().unwrap_or_else(|e| e));
    add_internal_function!(ctx, "exit", |code: i32| exit(code).unwrap_or(0));

    Ok(())
}

pub fn get_env() -> Result<String, String> {
    let env_vars: std::collections::HashMap<String, String> = std::env::vars().collect();
    Ok(serde_json::to_string(&env_vars).unwrap())
}

pub fn get_argv() -> Result<String, String> {
    let mut args: Vec<String> = std::env::args().collect();

    // Convert the first argument (executable path) to absolute path
    if !args.is_empty() {
        if let Ok(exe_path) = std::env::current_exe() {
            args[0] = exe_path.to_string_lossy().to_string();
        }
    }

    // Convert argv[1] (script path) to absolute canonical path
    if let Some(script_path) = args.get_mut(1) {
        if !script_path.is_empty() {
            if let Ok(canonical) = Path::new(script_path).canonicalize() {
                let path_str = canonical.display().to_string();
                *script_path = path_str
                    .strip_prefix(r"\\?\")
                    .unwrap_or(&path_str)
                    .to_string();
            }
        }
    }

    Ok(serde_json::to_string(&args).unwrap())
}

pub fn exit(code: i32) -> Result<i32, String> {
    std::process::exit(code);
}
