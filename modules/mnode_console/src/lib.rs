use mnode_utils::add_internal_function;
use rquickjs::{Ctx, Result};

pub fn init(ctx: &Ctx<'_>) -> Result<()> {
    add_internal_function!(ctx, "print", |msg: String| {
        println!("{}", msg);
    });

    ctx.eval::<(), _>(include_str!("console.js"))?;

    Ok(())
}
