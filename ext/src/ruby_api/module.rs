use std::{
    mem::{transmute, MaybeUninit},
    ops::Deref,
    os::raw::c_void,
};

use super::{engine::Engine, root};
use crate::{
    error,
    helpers::{nogvl, Tmplock},
};
use magnus::{class, function, method, rb_sys::AsRawValue, Error, Module as _, Object, RString};
use rb_sys::{
    rb_str_locktmp, rb_str_unlocktmp, tracking_allocator::ManuallyTracked, RSTRING_LEN, RSTRING_PTR,
};
use wasmtime::Module as ModuleImpl;

/// @yard
/// Represents a WebAssembly module.
/// @see https://docs.rs/wasmtime/latest/wasmtime/struct.Module.html Wasmtime's Rust doc
#[derive(Clone)]
#[magnus::wrap(class = "Wasmtime::Module", size, free_immediately, frozen_shareable)]
pub struct Module {
    inner: ModuleImpl,
    _track_memory_usage: ManuallyTracked<()>,
}

// Needed for ManuallyTracked
unsafe impl Send for Module {}

impl Module {
    /// @yard
    /// @def new(engine, wat_or_wasm)
    /// @param engine [Wasmtime::Engine]
    /// @param wat_or_wasm [String] The String of WAT or Wasm.
    /// @return [Wasmtime::Module]
    pub fn new(engine: &Engine, wat_or_wasm: RString) -> Result<Self, Error> {
        let eng = engine.get();
        let (locked_slice, _locked_slice_guard) = unsafe { wat_or_wasm.as_locked_slice() }?;
        let module = nogvl(|| ModuleImpl::new(eng, locked_slice))
            .map_err(|e| error!("Could not build module: {}", e))?;

        Ok(module.into())
    }

    /// @yard
    /// @def from_file(engine, path)
    /// @param engine [Wasmtime::Engine]
    /// @param path [String]
    /// @return [Wasmtime::Module]
    pub fn from_file(engine: &Engine, path: RString) -> Result<Self, Error> {
        let eng = engine.get();
        let (path, _locked_str_guard) = unsafe { path.as_locked_str()? };
        // SAFETY: this string is immediately copied and never moved off the stack
        let module = nogvl(|| ModuleImpl::from_file(eng, path))
            .map_err(|e| error!("Could not build module from file: {}", e))?;

        Ok(module.into())
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
            .map(Into::into)
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
            .map(Into::into)
            .map_err(|e| error!("Could not deserialize module from file: {}", e))
    }

    /// @yard
    /// Serialize the module.
    /// @return [String]
    /// @see .deserialize
    pub fn serialize(&self) -> Result<RString, Error> {
        let module = self.get();
        let bytes = nogvl(|| module.serialize());

        bytes
            .map(|bytes| RString::from_slice(&bytes))
            .map_err(|e| error!("{:?}", e))
    }

    pub fn get(&self) -> &ModuleImpl {
        &self.inner
    }
}

impl From<ModuleImpl> for Module {
    fn from(inner: ModuleImpl) -> Self {
        let size = inner.image_range().len();

        Self {
            inner,
            _track_memory_usage: ManuallyTracked::new(size),
        }
    }
}

pub fn init() -> Result<(), Error> {
    let class = root().define_class("Module", class::object())?;

    class.define_singleton_method("new", function!(Module::new, 2))?;
    class.define_singleton_method("from_file", function!(Module::from_file, 2))?;
    class.define_singleton_method("deserialize", function!(Module::deserialize, 2))?;
    class.define_singleton_method("deserialize_file", function!(Module::deserialize_file, 2))?;
    class.define_method("serialize", method!(Module::serialize, 0))?;

    Ok(())
}
