use magnus::{define_module, memoize, Error, RModule};

mod config;
mod convert;
mod engine;
mod errors;
mod export;
mod func;
mod func_type;
mod instance;
mod module;
mod params;
mod store;

/// The "Wasmtime" Ruby module.
pub fn root() -> RModule {
    *memoize!(RModule: define_module("Wasmtime").unwrap())
}

pub fn init() -> Result<(), Error> {
    config::init()?;
    engine::init()?;
    module::init()?;
    store::init()?;
    instance::init()?;
    export::init()?;
    func::init()?;
    func_type::init()?;
    Ok(())
}
