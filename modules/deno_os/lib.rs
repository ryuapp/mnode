// Copyright 2018-2025 the Deno authors. MIT license.
use rquickjs::{Ctx, Module};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{Read, Seek};
use utils::add_internal_function;

pub fn init(ctx: &Ctx<'_>) -> rquickjs::Result<()> {
    setup_internal(ctx).map_err(|_| rquickjs::Error::Unknown)?;
    let module =
        Module::evaluate(ctx.clone(), "deno_os", include_str!("deno_os.js")).map_err(|e| {
            eprintln!("deno_os.js eval error: {:?}", e);
            e
        })?;
    module.finish::<()>()?;
    Ok(())
}

fn setup_internal(ctx: &Ctx) -> Result<(), Box<dyn std::error::Error>> {
    // Deno.exit
    add_internal_function!(ctx, "exit", |code: Option<i32>| -> i32 {
        let exit_code = code.unwrap_or(0);
        std::process::exit(exit_code);
    });

    // Deno.env
    {
        ctx.eval::<(), _>("globalThis[Symbol.for('mdeno.internal')].env = {};")?;
        add_internal_function!(ctx, "env.get", |key: String| -> Option<String> {
            env::var(&key).ok()
        });
        add_internal_function!(ctx, "env.set", |key: String, value: String| {
            unsafe {
                env::set_var(&key, value);
            }
        });
        add_internal_function!(ctx, "env.delete", |key: String| {
            unsafe {
                env::remove_var(&key);
            }
        });
        add_internal_function!(ctx, "env.has", |key: String| -> bool {
            env::var(&key).is_ok()
        });
        add_internal_function!(ctx, "env.toObject", || -> HashMap<String, String> {
            env::vars().collect()
        });
    }

    // Deno.noColor - store in internal namespace
    let no_color = env::var("NO_COLOR").is_ok();
    let script = format!(
        "globalThis[Symbol.for('mdeno.internal')].noColor = {};",
        no_color
    );
    ctx.eval::<(), _>(script)?;

    // Deno.build - derive target triple and vendor from cfg! macros
    let (os, arch, target, vendor) = if cfg!(target_os = "windows") {
        let arch = if cfg!(target_arch = "x86_64") {
            "x86_64"
        } else if cfg!(target_arch = "aarch64") {
            "aarch64"
        } else if cfg!(target_arch = "x86") {
            "x86"
        } else {
            "unknown"
        };
        let target = format!("{}-pc-windows-msvc", arch);
        ("windows", arch, target, "pc")
    } else if cfg!(target_os = "macos") {
        let arch = if cfg!(target_arch = "x86_64") {
            "x86_64"
        } else if cfg!(target_arch = "aarch64") {
            "aarch64"
        } else {
            "unknown"
        };
        let target = format!("{}-apple-darwin", arch);
        ("darwin", arch, target, "apple")
    } else if cfg!(target_os = "linux") {
        let arch = if cfg!(target_arch = "x86_64") {
            "x86_64"
        } else if cfg!(target_arch = "aarch64") {
            "aarch64"
        } else if cfg!(target_arch = "arm") {
            "arm"
        } else if cfg!(target_arch = "x86") {
            "x86"
        } else {
            "unknown"
        };
        let target = if cfg!(target_env = "musl") {
            format!("{}-unknown-linux-musl", arch)
        } else {
            format!("{}-unknown-linux-gnu", arch)
        };
        ("linux", arch, target, "unknown")
    } else if cfg!(target_os = "freebsd") {
        let arch = if cfg!(target_arch = "x86_64") {
            "x86_64"
        } else {
            "unknown"
        };
        let target = format!("{}-unknown-freebsd", arch);
        ("freebsd", arch, target, "unknown")
    } else {
        (
            "unknown",
            "unknown",
            "unknown-unknown-unknown".to_string(),
            "unknown",
        )
    };

    // Determine if this is a standalone build by checking for magic string in the executable
    let standalone = is_standalone_binary();

    let build_info = format!(
        r#"globalThis[Symbol.for('mdeno.internal')].build = {{
  os: "{}",
  arch: "{}",
  target: "{}",
  vendor: "{}",
  standalone: {}
}};"#,
        os, arch, target, vendor, standalone
    );
    ctx.eval::<(), _>(build_info)?;

    Ok(())
}

/// Check if the current executable is a standalone binary by looking for the magic string "md3n04cl1"
fn is_standalone_binary() -> bool {
    // Get the current executable path
    let Ok(exe_path) = env::current_exe() else {
        return false;
    };

    // Try to open and read the executable file
    let Ok(mut file) = fs::File::open(&exe_path) else {
        return false;
    };

    // Read the last 1MB of the file to look for the magic string
    // This is an optimization to avoid reading the entire file
    let file_size = match file.metadata() {
        Ok(metadata) => metadata.len() as usize,
        Err(_) => return false,
    };

    let search_size = std::cmp::min(1024 * 1024, file_size);
    let offset = file_size.saturating_sub(search_size);

    if let Err(_) = file.seek(std::io::SeekFrom::Start(offset as u64)) {
        return false;
    }

    let mut buffer = vec![0; search_size];
    let bytes_read = match file.read(&mut buffer) {
        Ok(n) => n,
        Err(_) => return false,
    };

    // Search for the magic string "md3n04cl1"
    buffer[..bytes_read].windows(9).any(|w| w == b"md3n04cl1")
}
