use quickjs_rusty::{Callback, Context};

pub fn load_setup() -> &'static str {
    include_str!("setup.js")
}

pub fn add_internal_function<P>(
    context: &Context,
    internal_name: &str,
    callback: impl Callback<P> + 'static,
) -> Result<(), Box<dyn std::error::Error>> {
    let temp_name = format!("__mnode_internal_{}", internal_name.replace(".", "_"));
    let internal_path = format!("globalThis[Symbol.for('mnode.internal')].{}", internal_name);

    context.add_callback(&temp_name, callback)?;
    context.eval(
        &format!(
            "{} = globalThis.{}; delete globalThis.{};",
            internal_path, temp_name, temp_name
        ),
        false,
    )?;
    Ok(())
}
