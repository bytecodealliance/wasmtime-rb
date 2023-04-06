use super::{engine::Engine, root};
use crate::error;
use magnus::{function, method, Error, Module as _, Object, RString};
use wasmtime::Module as ModuleImpl;

/// @yard
/// Represents a WebAssembly module.
/// @see https://docs.rs/wasmtime/latest/wasmtime/struct.Module.html Wasmtime's Rust doc
#[derive(Clone)]
#[magnus::wrap(class = "Wasmtime::Module", size, free_immediately, frozen_shareable)]
pub struct Module {
    inner: ModuleImpl,
}

impl Module {
    /// @yard
    /// @def new(engine, wat_or_wasm)
    /// @param engine [Wasmtime::Engine]
    /// @param wat_or_wasm [String] The String of WAT or Wasm.
    /// @return [Wasmtime::Module]
    pub fn new(engine: &Engine, wat_or_wasm: RString) -> Result<Self, Error> {
        let eng = engine.get();
        // SAFETY: this string is immediately copied and never moved off the stack
        let module = ModuleImpl::new(eng, unsafe { wat_or_wasm.as_slice() })
            .map_err(|e| error!("Could not build module: {}", e))?;

        Ok(Self { inner: module })
    }

    /// @yard
    /// @def from_file(engine, path)
    /// @param engine [Wasmtime::Engine]
    /// @param path [String]
    /// @return [Wasmtime::Module]
    pub fn from_file(engine: &Engine, path: RString) -> Result<Self, Error> {
        let eng = engine.get();
        // SAFETY: this string is immediately copied and never moved off the stack
        let module = ModuleImpl::from_file(eng, unsafe { path.as_str()? })
            .map_err(|e| error!("Could not build module from file: {}", e))?;

        Ok(Self { inner: module })
    }

    /// @yard
    /// Instantiates a serialized module coming from either {#serialize} or {Wasmtime::Engine#precompile_module}.
    ///
    /// The engine serializing and the engine deserializing must:
    /// * have the same configuration
    /// * be of the same gem version
    ///
    /// @def deserialize(engine, compiled)
    /// @param engine [Wasmtime::Engine]
    /// @param compiled [String] String obtained with either {Wasmtime::Engine#precompile_module} or {#serialize}.
    /// @return [Wasmtime::Module]
    pub fn deserialize(engine: &Engine, compiled: RString) -> Result<Self, Error> {
        // SAFETY: this string is immediately copied and never moved off the stack
        unsafe { ModuleImpl::deserialize(engine.get(), compiled.as_slice()) }
            .map(|module| Self { inner: module })
            .map_err(|e| error!("Could not deserialize module: {}", e))
    }

    /// @yard
    /// Instantiates a serialized module from a file.
    ///
    /// @def deserialize_file(engine, path)
    /// @param engine [Wasmtime::Engine]
    /// @param path [String]
    /// @return [Wasmtime::Module]
    /// @see .deserialize
    pub fn deserialize_file(engine: &Engine, path: RString) -> Result<Self, Error> {
        unsafe { ModuleImpl::deserialize_file(engine.get(), path.as_str()?) }
            .map(|module| Self { inner: module })
            .map_err(|e| error!("Could not deserialize module from file: {}", e))
    }

    /// @yard
    /// Serialize the module.
    /// @return [String]
    /// @see .deserialize
    pub fn serialize(&self) -> Result<RString, Error> {
        self.get()
            .serialize()
            .map(|bytes| RString::from_slice(&bytes))
            .map_err(|e| error!("{:?}", e))
    }

    pub fn get(&self) -> &ModuleImpl {
        &self.inner
    }
}

pub fn init() -> Result<(), Error> {
    let class = root().define_class("Module", Default::default())?;

    class.define_singleton_method("new", function!(Module::new, 2))?;
    class.define_singleton_method("from_file", function!(Module::from_file, 2))?;
    class.define_singleton_method("deserialize", function!(Module::deserialize, 2))?;
    class.define_singleton_method("deserialize_file", function!(Module::deserialize_file, 2))?;
    class.define_method("serialize", method!(Module::serialize, 0))?;

    Ok(())
}
