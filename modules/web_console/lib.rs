use rquickjs::{Ctx, Module, Result};
use utils::add_internal_function;

pub fn init(ctx: &Ctx<'_>) -> Result<()> {
    add_internal_function!(ctx, "print", |msg: String| {
        println!("{}", msg);
    });

    let module = Module::evaluate(ctx.clone(), "web_console", include_str!("console.js"))?;
    module.finish::<()>()?;

    Ok(())
}
