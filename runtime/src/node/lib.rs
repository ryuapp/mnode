use rquickjs::loader::{Loader, Resolver};
use rquickjs::{Ctx, Error, Module, Result};

pub fn load_fs() -> &'static str {
    include_str!("fs/fs.js")
}

pub fn load_process() -> &'static str {
    include_str!("process/process.js")
}

pub struct NodeResolver;

impl Resolver for NodeResolver {
    fn resolve(&mut self, _ctx: &Ctx, _base: &str, name: &str) -> Result<String> {
        match name {
            "node:fs" | "node:process" => Ok(name.to_string()),
            _ => Err(Error::new_resolving(name, "Unknown node module")),
        }
    }
}

pub struct NodeLoader;

impl Loader for NodeLoader {
    fn load<'js>(&mut self, ctx: &Ctx<'js>, name: &str) -> Result<Module<'js>> {
        let source = match name {
            "node:fs" => load_fs(),
            "node:process" => load_process(),
            _ => return Err(Error::new_loading(name)),
        };

        Module::declare(ctx.clone(), name, source)
    }
}
