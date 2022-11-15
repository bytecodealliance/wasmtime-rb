use super::{config::Config, root};
use crate::error;
use magnus::{function, method, scan_args, Error, Module, Object, RString, Value};
use wasmtime::Engine as EngineImpl;

/// @yard
/// Represents a Wasmtime execution engine.
/// @see https://docs.rs/wasmtime/latest/wasmtime/struct.Engine.html Wasmtime's Rust doc
#[derive(Clone)]
#[magnus::wrap(class = "Wasmtime::Engine")]
pub struct Engine {
    inner: EngineImpl,
}

impl Engine {
    /// @yard
    /// @def new(config)
    /// @param config [Configuration]
    pub fn new(args: &[Value]) -> Result<Self, Error> {
        let args = scan_args::scan_args::<(), (Option<Value>,), (), (), (), ()>(args)?;
        let (config,) = args.optional;
        let config = config.and_then(|v| if v.is_nil() { None } else { Some(v) });
        let inner = match config {
            Some(config) => EngineImpl::new(&config.try_convert::<&Config>()?.get())
                .map_err(|e| error!("{}", e))?,
            None => EngineImpl::default(),
        };

        Ok(Self { inner })
    }

    pub fn get(&self) -> &EngineImpl {
        &self.inner
    }

    pub fn is_equal(&self, other: &Engine) -> bool {
        EngineImpl::same(self.get(), other.get())
    }

    /// @yard
    /// AoT compile a WebAssembly text or WebAssembly binary module for later use.
    ///
    /// The compiled module can be instantiated using {Module.deserialize}.
    ///
    /// @def precompile_module(wat_or_wasm)
    /// @param wat_or_wasm [String] The String of WAT or Wasm.
    /// @return [String] Binary String of the compiled module.
    /// @see Module.deserialize
    pub fn precompile_module(&self, wat_or_wasm: RString) -> Result<RString, Error> {
        self.inner
            .precompile_module(unsafe { wat_or_wasm.as_slice() })
            .map(|bytes| RString::from_slice(&bytes))
            .map_err(|e| error!("{}", e.to_string()))
    }
}

pub fn init() -> Result<(), Error> {
    let class = root().define_class("Engine", Default::default())?;

    class.define_singleton_method("new", function!(Engine::new, -1))?;

    class.define_method("==", method!(Engine::is_equal, 1))?;
    class.define_method("precompile_module", method!(Engine::precompile_module, 1))?;

    Ok(())
}
