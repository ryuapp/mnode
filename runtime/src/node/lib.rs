use anyhow::anyhow;
use quickjs_rusty::Context;

pub fn load_fs() -> &'static str {
    include_str!("fs/fs.js")
}

pub fn load_process() -> &'static str {
    include_str!("process/process.js")
}

pub fn set_module_loader(context: &Context) -> Result<(), Box<dyn std::error::Error>> {
    context.set_module_loader(
        Box::new(
            |name: &str, _opaque: *mut std::ffi::c_void| -> Result<String, anyhow::Error> {
                match name {
                    "node:fs" => Ok(load_fs().to_string()),
                    "node:process" => Ok(load_process().to_string()),
                    _ => Err(anyhow!("Module not found: {}", name)),
                }
            },
        ),
        None,
        std::ptr::null_mut(),
    );
    Ok(())
}
