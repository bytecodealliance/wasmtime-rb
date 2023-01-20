#![allow(rustdoc::broken_intra_doc_links)]
#![allow(rustdoc::invalid_html_tags)]
#![allow(rustdoc::bare_urls)]
#![allow(rustdoc::invalid_rust_codeblocks)]
use magnus::{define_module, function, memoize, Error, RModule, RString};

pub mod caller;
pub mod config;
pub mod convert;
pub mod engine;
pub mod errors;
pub mod externals;
pub mod func;
pub mod global;
pub mod instance;
pub mod linker;
pub mod memory;
pub mod module;
pub mod params;
pub mod store;
pub mod table;
pub mod trap;
pub mod wasi_ctx_builder;

pub use caller::Caller;
pub use engine::Engine;
pub use func::Func;
pub use instance::Instance;
pub use linker::Linker;
pub use memory::Memory;
pub use module::Module;
pub use params::Params;
pub use store::Store;
pub use trap::Trap;
pub use wasi_ctx_builder::WasiCtxBuilder;

/// The "Wasmtime" Ruby module.
pub fn root() -> RModule {
    *memoize!(RModule: define_module("Wasmtime").unwrap())
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

pub fn init() -> Result<(), Error> {
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
    memory::init()?;
    linker::init()?;
    externals::init()?;
    wasi_ctx_builder::init()?;
    table::init()?;
    global::init()?;

    Ok(())
}
