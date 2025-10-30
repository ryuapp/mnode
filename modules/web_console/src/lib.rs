use rquickjs::{Ctx, Result};
use utils::add_internal_function;

pub fn init(ctx: &Ctx<'_>) -> Result<()> {
    add_internal_function!(ctx, "print", |msg: String| {
        println!("{}", msg);
    });

    ctx.eval::<(), _>(include_str!("console.js"))?;

    Ok(())
}
