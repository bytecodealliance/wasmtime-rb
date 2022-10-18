#![allow(rustdoc::broken_intra_doc_links)]
#![allow(rustdoc::invalid_html_tags)]
use magnus::{define_module, memoize, Error, RModule};

mod config;
mod convert;
mod engine;
mod errors;
mod func;
mod func_type;
mod instance;
mod linker;
mod macros;
mod memory;
mod memory_type;
mod module;
mod params;
mod static_id;
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
    func::init()?;
    func_type::init()?;
    memory_type::init()?;
    memory::init()?;
    linker::init()?;
    Ok(())
}
