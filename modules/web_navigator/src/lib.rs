use rquickjs::Ctx;
use std::error::Error;

pub fn init(ctx: &Ctx<'_>) -> rquickjs::Result<()> {
    setup_internal(ctx).map_err(|_| rquickjs::Error::Unknown)?;
    ctx.eval::<(), _>(include_str!("navigator.js"))
}

fn setup_internal(ctx: &Ctx) -> Result<(), Box<dyn Error>> {
    let platform = if cfg!(target_os = "macos") {
        "MacIntel"
    } else if cfg!(windows) {
        "Win32"
    } else if cfg!(target_os = "linux") {
        if cfg!(target_arch = "x86_64") {
            "Linux x86_64"
        } else if cfg!(target_arch = "aarch64") {
            "Linux armv81"
        } else {
            return Ok(());
        }
    } else {
        return Ok(());
    };

    ctx.eval::<(), _>(format!(
        "globalThis[Symbol.for('mnode.internal')].platform = '{}';",
        platform
    ))?;

    Ok(())
}
