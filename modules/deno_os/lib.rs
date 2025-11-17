// Copyright 2018-2025 the Deno authors. MIT license.
use rquickjs::Ctx;
use std::collections::HashMap;
use std::env;
use utils::add_internal_function;

pub fn init(ctx: &Ctx<'_>) -> rquickjs::Result<()> {
    setup_internal(ctx).map_err(|_| rquickjs::Error::Unknown)?;
    ctx.eval::<(), _>(include_str!("deno_os.js"))
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

    // Deno.noColor
    let no_color = env::var("NO_COLOR").is_ok();
    ctx.globals()
        .set("__mdeno_no_color", no_color)
        .map_err(|e| format!("Failed to set __mdeno_no_color: {}", e))?;

    Ok(())
}
