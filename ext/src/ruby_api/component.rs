mod convert;
mod func;
mod instance;
mod linker;
mod wasi_command;

use super::root;
use magnus::{
    class, class::RClass, function, method, prelude::*, r_string::RString, value::Lazy, Error,
    Module, Object, RModule, Ruby,
};
use rb_sys::tracking_allocator::ManuallyTracked;
use wasmtime::component::Component as ComponentImpl;

pub use func::Func;
pub use instance::Instance;
pub use wasi_command::WasiCommand;

pub fn component_namespace(ruby: &Ruby) -> RModule {
    static COMPONENT_NAMESPACE: Lazy<RModule> =
        Lazy::new(|_| root().define_module("Component").unwrap());
    ruby.get_inner(&COMPONENT_NAMESPACE)
}

use crate::{
    error,
    helpers::{nogvl, Tmplock},
    Engine,
};
/// @yard
/// @rename Wasmtime::Component::Component
/// Represents a WebAssembly component.
/// @see https://docs.rs/wasmtime/latest/wasmtime/component/struct.Component.html Wasmtime's Rust doc
#[magnus::wrap(
    class = "Wasmtime::Component::Component",
    size,
    free_immediately,
    frozen_shareable
)]
pub struct Component {
    inner: ComponentImpl,
    _track_memory_usage: ManuallyTracked<()>,
}

// Needed for ManuallyTracked
unsafe impl Send for Component {}

impl Component {
    /// @yard
    /// Creates a new component from the given binary data.
    /// @def new(engine, wat_or_wasm)
    /// @param engine [Wasmtime::Engine]
    /// @param wat_or_wasm [String] The String of WAT or Wasm.
    /// @return [Wasmtime::Component::Component]
    pub fn new(engine: &Engine, wat_or_wasm: RString) -> Result<Self, Error> {
        let eng = engine.get();
        let (locked_slice, _locked_slice_guard) = wat_or_wasm.as_locked_slice()?;
        let component = nogvl(|| ComponentImpl::new(eng, locked_slice))
            .map_err(|e| error!("Could not build component: {}", e))?;

        Ok(component.into())
    }

    /// @yard
    /// @def from_file(engine, path)
    /// @param engine [Wasmtime::Engine]
    /// @param path [String]
    /// @return [Wasmtime::Component::Component]
    pub fn from_file(engine: &Engine, path: RString) -> Result<Self, Error> {
        let eng = engine.get();
        let (path, _locked_str_guard) = path.as_locked_str()?;
        // SAFETY: this string is immediately copied and never moved off the stack
        let component = nogvl(|| ComponentImpl::from_file(eng, path))
            .map_err(|e| error!("Could not build component from file: {}", e))?;

        Ok(component.into())
    }

    /// @yard
    /// Instantiates a serialized component coming from either {#serialize} or {Wasmtime::Engine#precompile_component}.
    ///
    /// The engine serializing and the engine deserializing must:
    /// * have the same configuration
    /// * be of the same gem version
    ///
    /// @def deserialize(engine, compiled)
    /// @param engine [Wasmtime::Engine]
    /// @param compiled [String] String obtained with either {Wasmtime::Engine#precompile_component} or {#serialize}.
    /// @return [Wasmtime::Component::Component]
    pub fn deserialize(engine: &Engine, compiled: RString) -> Result<Self, Error> {
        // SAFETY: this string is immediately copied and never moved off the stack
        unsafe { ComponentImpl::deserialize(engine.get(), compiled.as_slice()) }
            .map(Into::into)
            .map_err(|e| error!("Could not deserialize component: {}", e))
    }

    /// @yard
    /// Instantiates a serialized component from a file.
    ///
    /// @def deserialize_file(engine, path)
    /// @param engine [Wasmtime::Engine]
    /// @param path [String]
    /// @return [Wasmtime::Component::Component]
    /// @see .deserialize
    pub fn deserialize_file(engine: &Engine, path: RString) -> Result<Self, Error> {
        unsafe { ComponentImpl::deserialize_file(engine.get(), path.as_str()?) }
            .map(Into::into)
            .map_err(|e| error!("Could not deserialize component from file: {}", e))
    }

    /// @yard
    /// Serialize the component.
    /// @return [String]
    /// @see .deserialize
    pub fn serialize(&self) -> Result<RString, Error> {
        let bytes = self.get().serialize();

        bytes
            .map(|bytes| RString::from_slice(&bytes))
            .map_err(|e| error!("{:?}", e))
    }

    pub fn get(&self) -> &ComponentImpl {
        &self.inner
    }
}

impl From<ComponentImpl> for Component {
    fn from(inner: ComponentImpl) -> Self {
        let range = inner.image_range();
        let start = range.start;
        let end = range.end;

        assert!(end > start);
        let size = unsafe { end.offset_from(start) };

        Self {
            inner,
            _track_memory_usage: ManuallyTracked::new(size as usize),
        }
    }
}

mod bundled {
    include!(concat!(env!("OUT_DIR"), "/bundled/component.rs"));
}

pub fn init(ruby: &Ruby) -> Result<(), Error> {
    bundled::init()?;

    let namespace = component_namespace(ruby);

    let class = namespace.define_class("Component", class::object())?;
    class.define_singleton_method("new", function!(Component::new, 2))?;
    class.define_singleton_method("from_file", function!(Component::from_file, 2))?;
    class.define_singleton_method("deserialize", function!(Component::deserialize, 2))?;
    class.define_singleton_method(
        "deserialize_file",
        function!(Component::deserialize_file, 2),
    )?;
    class.define_method("serialize", method!(Component::serialize, 0))?;

    linker::init(ruby, &namespace)?;
    instance::init(ruby, &namespace)?;
    func::init(ruby, &namespace)?;
    convert::init(ruby)?;
    wasi_command::init(ruby, &namespace)?;

    Ok(())
}
