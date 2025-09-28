use quickjs_rusty::Context;

pub fn setup(context: &Context) -> Result<(), Box<dyn std::error::Error>> {
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

    context.eval(
        &format!(
            "globalThis[Symbol.for('mnode.internal')].platform = '{}';",
            platform
        ),
        false,
    )?;

    Ok(())
}
