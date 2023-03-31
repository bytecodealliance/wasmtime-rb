use std::sync::Mutex;
use super::{config::hash_to_config, root};
use crate::error;
use magnus::{function, method, scan_args, Error, Module, Object, RHash, RString, Value};
use wasmtime::Engine as EngineImpl;

#[cfg(feature = "tokio")]
lazy_static::lazy_static! {
    static ref TOKIO_RT: tokio::runtime::Runtime = tokio::runtime::Builder::new_multi_thread()
        .thread_name("wasmtime-engine-timers")
        .worker_threads(1)
        .enable_io()
        .build()
        .unwrap();
}

/// @yard
/// Represents a Wasmtime execution engine.
/// @see https://docs.rs/wasmtime/latest/wasmtime/struct.Engine.html Wasmtime's Rust doc
#[magnus::wrap(class = "Wasmtime::Engine", free_immediately, frozen_shareable)]
pub struct Engine {
    inner: EngineImpl,

    #[cfg(feature = "tokio")]
    timer_task: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

#[cfg(feature = "tokio")]
impl Drop for Engine {
    fn drop(&mut self) {
        self.stop_epoch_interval()
    }
}

impl Engine {
    /// @yard
    /// @def new(config = {})
    /// @param config [Hash] The engine's config.
    ///   See the {https://docs.rs/wasmtime/latest/wasmtime/struct.Engine.html +Config+‘s Rust doc} for detailed description of
    ///   the different options and the defaults.
    /// @option config [Boolean] :debug_info
    /// @option config [Boolean] :wasm_backtrace_details
    /// @option config [Boolean] :native_unwind_info
    /// @option config [Boolean] :consume_fuel
    /// @option config [Boolean] :epoch_interruption
    /// @option config [Integer] :max_wasm_stack
    /// @option config [Boolean] :wasm_threads
    /// @option config [Boolean] :wasm_multi_memory
    /// @option config [Boolean] :wasm_memory64
    /// @option config [Boolean] :parallel_compilation
    /// @option config [Symbol] :cranelift_opt_level One of +none+, +speed+, +speed_and_size+.
    /// @option config [Symbol] :profiler One of +none+, +jitdump+, +vtune+.
    /// @option config [String] :target
    ///
    /// @see https://docs.rs/wasmtime/latest/wasmtime/struct.Engine.html
    ///     Wasmtime's Rust doc for details of the configuration options.
    pub fn new(args: &[Value]) -> Result<Self, Error> {
        let args = scan_args::scan_args::<(), (Option<Value>,), (), (), (), ()>(args)?;
        let (config,) = args.optional;
        let config = config.and_then(|v| if v.is_nil() { None } else { Some(v) });
        let inner = match config {
            Some(config) => {
                let config = config.try_convert::<RHash>().and_then(hash_to_config)?;

                EngineImpl::new(&config).map_err(|e| error!("{}", e))?
            }
            None => EngineImpl::default(),
        };

        Ok(Self {
            inner,
            #[cfg(feature = "tokio")]
            timer_task: Default::default(),
        })
    }

    /// @yard
    /// Starts a timer that will increment the engine's epoch every +milliseconds+.
    /// Waits +milliseconds+ before incrementing for the first time.
    ///
    /// If a prior timer was started, it will be stopped.
    /// @def start_epoch_interval(milliseconds)
    /// @param milliseconds [Integer]
    /// @return [nil]
    #[cfg(feature = "tokio")]
    pub fn start_epoch_interval(&self, milliseconds: u64) {
        self.stop_epoch_interval();
        let engine = self.inner.clone();

        let handle = TOKIO_RT.spawn(async move {
            let tick_every = tokio::time::Duration::from_millis(milliseconds);
            let mut interval = async_timer::Interval::platform_new(tick_every);

            loop {
                interval.wait().await;
                engine.increment_epoch();
            }
        });

        *self.timer_task.lock().unwrap() = Some(handle);
    }

    /// @yard
    /// Stops a previously started timer with {#start_epoch_interval}.
    /// Does nothing if there is no running timer.
    /// @return [nil]
    #[cfg(feature = "tokio")]
    pub fn stop_epoch_interval(&self) {
        let maybe_handle = self.timer_task.lock().unwrap().take();

        if let Some(handle) = maybe_handle {
            handle.abort();
        }
    }

    /// @yard
    /// Manually increment the engine's epoch.
    /// Note: this cannot be used from a different thread while WebAssembly is
    /// running because the Global VM lock (GVL) is not released.
    /// Using {#start_epoch_interval} is recommended because it sidesteps the GVL.
    /// @return [nil]
    pub fn increment_epoch(&self) {
        self.inner.increment_epoch();
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

    pub fn get(&self) -> &EngineImpl {
        &self.inner
    }
}

pub fn init() -> Result<(), Error> {
    let class = root().define_class("Engine", Default::default())?;

    class.define_singleton_method("new", function!(Engine::new, -1))?;

    #[cfg(feature = "tokio")]
    class.define_method(
        "start_epoch_interval",
        method!(Engine::start_epoch_interval, 1),
    )?;

    #[cfg(feature = "tokio")]
    class.define_method(
        "stop_epoch_interval",
        method!(Engine::stop_epoch_interval, 0),
    )?;
    class.define_method("increment_epoch", method!(Engine::increment_epoch, 0))?;
    class.define_method("==", method!(Engine::is_equal, 1))?;
    class.define_method("precompile_module", method!(Engine::precompile_module, 1))?;

    Ok(())
}
