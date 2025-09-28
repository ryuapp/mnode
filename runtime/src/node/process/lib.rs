use crate::internal::add_internal_function;
use quickjs_rusty::Context;
use std::sync::OnceLock;

static SCRIPT_PATH: OnceLock<String> = OnceLock::new();

pub fn setup(context: &Context, script_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    SCRIPT_PATH.get_or_init(|| script_path.to_string());

    add_internal_function(context, "getEnv", get_env)?;
    add_internal_function(context, "getArgv", get_argv)?;
    add_internal_function(context, "exit", exit)?;
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

    // Replace argv[1] with the absolute script path if available
    if let Some(script_path) = SCRIPT_PATH.get() {
        if !script_path.is_empty() {
            if args.len() > 1 {
                args[1] = script_path.clone();
            } else {
                args.push(script_path.clone());
            }
        }
    }

    Ok(serde_json::to_string(&args).unwrap())
}

pub fn exit(code: i32) -> Result<i32, String> {
    std::process::exit(code);
}
