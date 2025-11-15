// Copyright 2018-2025 the Deno authors. MIT license.
use rquickjs::Ctx;
use utils::add_internal_function;

pub fn init(ctx: &Ctx<'_>) -> rquickjs::Result<()> {
    setup_internal(ctx).map_err(|_| rquickjs::Error::Unknown)?;
    ctx.eval::<(), _>(include_str!("deno_os.js"))
}

fn setup_internal(ctx: &Ctx) -> Result<(), Box<dyn std::error::Error>> {
    add_internal_function!(ctx, "exit", |code: Option<i32>| -> i32 {
        let exit_code = code.unwrap_or(0);
        std::process::exit(exit_code);
    });

    Ok(())
}
