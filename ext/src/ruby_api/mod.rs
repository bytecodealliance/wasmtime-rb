#![allow(rustdoc::broken_intra_doc_links)]
#![allow(rustdoc::invalid_html_tags)]
#![allow(rustdoc::bare_urls)]
#![allow(rustdoc::invalid_rust_codeblocks)]
// The `pub use` imports below need to be publicly exposed when the ruby_api
// feature is enabled, else they must be publicly exposed to the crate only
// (`pub(crate) use`). Allowing unused imports is easier and less repetitive.
// Also the feature is already correctly gated in lib.rs.
#![allow(unused_imports)]
use magnus::{function, value::Lazy, Error, RModule, RString, Ruby};

mod caller;
mod component;
mod config;
mod convert;
mod engine;
mod errors;
mod externals;
mod func;
mod global;
mod instance;
mod linker;
mod memory;
mod module;
mod params;
mod pooling_allocation_config;
mod store;
mod table;
mod trap;
mod wasi_config;

pub use caller::Caller;
pub use engine::Engine;
pub use func::Func;
pub use instance::Instance;
pub use linker::Linker;
pub use memory::Memory;
pub use module::Module;
pub use params::Params;
pub use pooling_allocation_config::PoolingAllocationConfig;
pub use store::Store;
pub use trap::Trap;
pub use wasi_config::WasiConfig;

/// The "Wasmtime" Ruby module.
pub fn root() -> RModule {
    static ROOT: Lazy<RModule> = Lazy::new(|ruby| ruby.define_module("Wasmtime").unwrap());
    let ruby = Ruby::get().unwrap();
    ruby.get_inner(&ROOT)
}

// This Struct is a placeholder for documentation, so that we can hang methods
// to it and have yard-rustdoc discover them.
/// @yard
/// @module
pub struct Wasmtime;
impl Wasmtime {
    /// @yard
    /// Converts a WAT +String+ into Wasm.
    /// @param wat [String]
    /// @def wat2wasm(wat)
    /// @return [String] The Wasm represented as a binary +String+.
    pub fn wat2wasm(wat: RString) -> Result<RString, Error> {
        wat::parse_str(unsafe { wat.as_str()? })
            .map(|bytes| RString::from_slice(bytes.as_slice()))
            .map_err(|e| crate::error!("{}", e))
    }
}

pub fn init(ruby: &Ruby) -> Result<(), Error> {
    let wasmtime = root();

    wasmtime.define_module_function("wat2wasm", function!(Wasmtime::wat2wasm, 1))?;

    errors::init()?;
    trap::init()?;
    engine::init()?;
    module::init()?;
    store::init()?;
    instance::init()?;
    func::init()?;
    caller::init()?;
    memory::init(ruby)?;
    linker::init()?;
    externals::init()?;
    wasi_config::init()?;
    table::init()?;
    global::init()?;
    pooling_allocation_config::init()?;
    component::init(ruby)?;

    Ok(())
}
