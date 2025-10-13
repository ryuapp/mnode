use rquickjs::{Ctx, Result};

pub trait ModuleDef {
    fn init(ctx: &Ctx<'_>) -> Result<()>;
    fn source() -> &'static str;
    fn name() -> &'static str;
}

#[macro_export]
macro_rules! add_internal_function {
    ($ctx:expr, $name:expr, $func:expr) => {{
        use rquickjs::function::Func;
        let temp_name = format!("__mnode_internal_{}", $name.replace('.', "_"));
        let internal_path = format!("globalThis[Symbol.for('mnode.internal')].{}", $name);

        let func = Func::from($func);
        $ctx.globals().set(temp_name.as_str(), func)?;
        $ctx.eval::<(), _>(format!(
            "{} = globalThis.{}; delete globalThis.{};",
            internal_path, temp_name, temp_name
        ))?
    }};
}
