use super::{config::Config, root};
use crate::error;
use lazy_static::lazy_static;
use magnus::{function, method, scan_args, Error, Module, Object, RString, Value};
use std::cell::RefCell;
use tokio::{runtime, task::JoinHandle, time};
use wasmtime::Engine as EngineImpl;

lazy_static! {
    static ref TOKIO_RT: runtime::Runtime = runtime::Builder::new_multi_thread()
        .thread_name("wasmtime-engine-timers")
        .worker_threads(1)
        .enable_time()
        .build()
        .unwrap();
}

/// @yard
/// Represents a Wasmtime execution engine.
/// @see https://docs.rs/wasmtime/latest/wasmtime/struct.Engine.html Wasmtime's Rust doc
#[magnus::wrap(class = "Wasmtime::Engine", free_immediately)]
pub struct Engine {
    inner: EngineImpl,
    timer_task: RefCell<Option<JoinHandle<()>>>,
}

impl Drop for Engine {
    fn drop(&mut self) {
        self.stop_epoch_interval()
    }
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

        Ok(Self {
            inner,
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
    pub fn start_epoch_interval(&self, milliseconds: u64) {
        self.stop_epoch_interval();
        let engine = self.inner.clone();

        let handle = TOKIO_RT.spawn(async move {
            let tick_every = time::Duration::from_millis(milliseconds);
            let start = time::Instant::now() + tick_every;
            let mut interval = time::interval_at(start, tick_every);

            loop {
                interval.tick().await;
                engine.increment_epoch();
            }
        });

        *self.timer_task.borrow_mut() = Some(handle);
    }

    /// @yard
    /// Stops a previously started timer with {#start_epoch_interval}.
    /// Does nothing if there is no running timer.
    /// @return [nil]
    pub fn stop_epoch_interval(&self) {
        if let Some(handle) = self.timer_task.take() {
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
    class.define_method(
        "start_epoch_interval",
        method!(Engine::start_epoch_interval, 1),
    )?;
    class.define_method(
        "stop_epoch_interval",
        method!(Engine::stop_epoch_interval, 0),
    )?;
    class.define_method("increment_epoch", method!(Engine::increment_epoch, 0))?;
    class.define_method("==", method!(Engine::is_equal, 1))?;
    class.define_method("precompile_module", method!(Engine::precompile_module, 1))?;

    Ok(())
}
