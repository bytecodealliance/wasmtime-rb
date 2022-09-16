use magnus::{define_module, memoize, Error, RModule};

mod config;
mod engine;
mod errors;
mod export;
mod instance;
mod module;
mod params;
mod store;
mod to_ruby_value;

/// The "Wasmtime" Ruby module.
pub fn root() -> RModule {
    *memoize!(RModule: define_module("Wasmtime").unwrap())
}

#[macro_export]
macro_rules! rtyped_data {
    ($value:expr) => {{
        magnus::RTypedData::from_value($value)
            .ok_or_else(|| error!("could not get inner typed data"))
    }};
}

pub fn init() -> Result<(), Error> {
    config::init()?;
    engine::init()?;
    module::init()?;
    store::init()?;
    instance::init()?;
    export::init()?;
    Ok(())
}
