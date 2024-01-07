use super::{
    config::{default_config, hash_to_config},
    root,
};
use crate::error;
use magnus::{
    class, function, method, prelude::*, scan_args, typed_data::Obj, value::LazyId, Error, Module,
    Object, RHash, RString, Ruby, TryConvert, Value,
};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    sync::Mutex,
};
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
///
/// @example Disabling parallel compilation
///    # Many Ruby servers use a pre-forking mechanism to allow parallel request
///    # processing. Unfortunately, this can causes processes to deadlock if you
///    # use parallel compilation to compile Wasm prior to calling
///    # `Process::fork`. To avoid this issue, any compilations that need to be
///    # done before forking need to disable the `parallel_compilation` option.
///
///    prefork_engine = Wasmtime::Engine.new(parallel_compilation: false)
///    wasm_module = Wasmtime::Module.new(prefork_engine, "(module)")
///
///    fork do
///      # We can enable parallel compilation now that we've forked.
///      engine = Wasmtime::Engine.new(parallel_compilation: true)
///      store = Wasmtime::Store.new(engine)
///      instance = Wasmtime::Instance.new(store, wasm_module)
///      # ...
///    end
///
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
    /// @option config [Boolean] :parallel_compilation (true) Whether compile WASM using multiple threads
    /// @option config [Boolean] :generate_address_map Configures whether compiled artifacts will contain information to map native program addresses back to the original wasm module. This configuration option is `true` by default. Disabling this feature can result in considerably smaller serialized modules.
    /// @option config [Symbol] :cranelift_opt_level One of +none+, +speed+, +speed_and_size+.
    /// @option config [Symbol] :profiler One of +none+, +jitdump+, +vtune+.
    /// @option config [Symbol] :strategy One of +auto+, +cranelift+, +winch+ (requires crate feature `winch` to be enabled)
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
                let config = RHash::try_convert(config).and_then(hash_to_config)?;

                EngineImpl::new(&config).map_err(|e| error!("{}", e))?
            }
            None => EngineImpl::new(&default_config()).map_err(|e| error!("{}", e))?,
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

    /// @yard
    /// If two engines have a matching {Engine.precompile_compatibility_key},
    /// then serialized modules from one engine can be deserialized by the
    /// other.
    /// @return [String] The hex formatted string that can be used to check precompiled module compatibility.
    pub fn precompile_compatibility_key(ruby: &Ruby, rb_self: Obj<Self>) -> Result<RString, Error> {
        static ID: LazyId = LazyId::new("precompile_compatibility_key");
        let ivar_id = LazyId::get_inner_with(&ID, ruby);

        if let Ok(cached) = rb_self.ivar_get::<_, RString>(ivar_id) {
            return Ok(cached);
        }

        let mut hasher = DefaultHasher::new();
        let engine = rb_self.inner.clone();
        engine.precompile_compatibility_hash().hash(&mut hasher);
        let hex_encoded = format!("{:x}", hasher.finish());
        let key = RString::new(&hex_encoded);
        key.freeze();

        rb_self.ivar_set(ivar_id, key)?;

        Ok(key)
    }

    pub fn get(&self) -> &EngineImpl {
        &self.inner
    }
}

pub fn init() -> Result<(), Error> {
    let class = root().define_class("Engine", class::object())?;

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
    class.define_method(
        "precompile_compatibility_key",
        method!(Engine::precompile_compatibility_key, 0),
    )?;

    Ok(())
}
