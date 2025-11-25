use super::errors::wasi_exit_error;
use super::{caller::Caller, engine::Engine, root, trap::Trap};
use crate::{define_rb_intern, error, WasiConfig};
use magnus::value::StaticSymbol;
use magnus::{
    class, function,
    gc::{Compactor, Marker},
    method, scan_args,
    typed_data::Obj,
    value::Opaque,
    DataTypeFunctions, Error, IntoValue, Module, Object, Ruby, TypedData, Value,
};
use magnus::{Class, RHash};
use rb_sys::tracking_allocator::{ManuallyTracked, TrackingAllocator};
use std::borrow::Borrow;
use std::cell::UnsafeCell;
use std::convert::TryFrom;
use wasmtime::{
    AsContext, AsContextMut, ResourceLimiter, Store as StoreImpl, StoreContext, StoreContextMut,
    StoreLimits, StoreLimitsBuilder,
};
use wasmtime_wasi::p1::WasiP1Ctx;
use wasmtime_wasi::{I32Exit, ResourceTable};
use wasmtime_wasi::{WasiCtx, WasiCtxView, WasiView};

define_rb_intern!(
    WASI_CONFIG => "wasi_config",
    WASI_P1_CONFIG => "wasi_p1_config",
    LIMITS => "limits",
);

pub struct StoreData {
    user_data: Value,
    wasi_p1: Option<WasiP1Ctx>,
    wasi: Option<WasiCtx>,
    refs: Vec<Value>,
    last_error: Option<Error>,
    store_limits: TrackingResourceLimiter,
    resource_table: ResourceTable,
}

impl StoreData {
    pub fn user_data(&self) -> Value {
        self.user_data
    }

    pub fn has_wasi_p1_ctx(&self) -> bool {
        self.wasi_p1.is_some()
    }

    pub fn has_wasi_ctx(&self) -> bool {
        self.wasi.is_some()
    }

    pub fn wasi_p1_ctx_mut(&mut self) -> &mut WasiP1Ctx {
        self.wasi_p1
            .as_mut()
            .expect("Store must have a WASI context")
    }

    pub fn retain(&mut self, value: Value) {
        self.refs.push(value);
    }

    pub fn set_error(&mut self, error: Error) {
        self.last_error = Some(error);
    }

    pub fn take_error(&mut self) -> Option<Error> {
        self.last_error.take()
    }

    pub fn mark(&self, marker: &Marker) {
        marker.mark_movable(self.user_data);

        if let Some(ref error) = self.last_error {
            if let Some(val) = error.value() {
                marker.mark(val);
            }
        }

        for value in self.refs.iter() {
            marker.mark_movable(*value);
        }
    }

    pub fn compact(&mut self, compactor: &Compactor) {
        self.user_data = compactor.location(self.user_data);

        for value in self.refs.iter_mut() {
            *value = compactor.location(*value);
        }
    }
}

/// @yard
/// Represents a WebAssembly store.
/// @see https://docs.rs/wasmtime/latest/wasmtime/struct.Store.html Wasmtime's Rust doc
#[derive(Debug, TypedData)]
#[magnus(class = "Wasmtime::Store", size, mark, compact, free_immediately)]
pub struct Store {
    inner: UnsafeCell<StoreImpl<StoreData>>,
}

impl DataTypeFunctions for Store {
    fn mark(&self, marker: &Marker) {
        self.context().data().mark(marker);
    }

    fn compact(&self, compactor: &Compactor) {
        self.context_mut().data_mut().compact(compactor);
    }
}

unsafe impl Send for Store {}
unsafe impl Send for StoreData {}

impl Store {
    /// @yard
    ///
    /// @def new(engine, data = nil, wasi_config: nil, wasi_p1_config: nil, limits: nil)
    /// @param engine [Wasmtime::Engine]
    ///   The engine for this store.
    /// @param data [Object]
    ///   The data attached to the store. Can be retrieved through {Wasmtime::Store#data} and {Wasmtime::Caller#data}.
    /// @param wasi_config [Wasmtime::WasiConfig]
    ///   The WASI config to use in this store.
    /// @param wasi_p1_config [Wasmtime::WasiConfig]
    ///   The WASI config to use in this store for WASI preview 1.
    /// @param limits [Hash]
    ///   See the {https://docs.rs/wasmtime/latest/wasmtime/struct.StoreLimitsBuilder.html +StoreLimitsBuilder+'s Rust doc}
    ///   for detailed description of the different options and the defaults.
    /// @option limits memory_size [Integer]
    ///   The maximum number of bytes a linear memory can grow to.
    /// @option limits table_elements [Integer]
    ///   The maximum number of elements in a table.
    /// @option limits instances [Integer]
    ///   The maximum number of instances that can be created for a Store.
    /// @option limits tables [Integer]
    ///   The maximum number of tables that can be created for a Store.
    /// @option limits memories [Integer]
    ///   The maximum number of linear memories that can be created for a Store.
    /// @return [Wasmtime::Store]
    ///
    /// @example
    ///   store = Wasmtime::Store.new(Wasmtime::Engine.new)
    ///
    /// @example
    ///   store = Wasmtime::Store.new(Wasmtime::Engine.new, {})
    pub fn new(args: &[Value]) -> Result<Self, Error> {
        let ruby = Ruby::get().unwrap();
        let args = scan_args::scan_args::<(&Engine,), (Option<Value>,), (), (), _, ()>(args)?;
        let kw = scan_args::get_kwargs::<
            _,
            (),
            (Option<&WasiConfig>, Option<&WasiConfig>, Option<RHash>),
            (),
        >(
            args.keywords,
            &[],
            &[*WASI_CONFIG, *WASI_P1_CONFIG, *LIMITS],
        )?;

        let (engine,) = args.required;
        let (user_data,) = args.optional;
        let user_data = user_data.unwrap_or_else(|| ().into_value_with(&ruby));
        let wasi_config = kw.optional.0;
        let wasi_p1_config = kw.optional.1;

        let wasi = wasi_config
            .map(|wasi_config| wasi_config.build(&ruby))
            .transpose()?;
        let wasi_p1 = wasi_p1_config
            .map(|wasi_config| wasi_config.build_p1(&ruby))
            .transpose()?;

        let limiter = match kw.optional.2 {
            None => StoreLimitsBuilder::new(),
            Some(limits) => hash_to_store_limits_builder(&ruby, limits)?,
        }
        .build();
        let limiter = TrackingResourceLimiter::new(limiter);

        let eng = engine.get();
        let store_data = StoreData {
            user_data,
            wasi_p1,
            wasi,
            refs: Default::default(),
            last_error: Default::default(),
            store_limits: limiter,
            resource_table: Default::default(),
        };
        let store = Self {
            inner: UnsafeCell::new(StoreImpl::new(eng, store_data)),
        };

        unsafe { &mut *store.inner.get() }.limiter(|data| &mut data.store_limits);

        Ok(store)
    }

    /// @yard
    /// @return [Object] The passed in value in {.new}
    pub fn data(&self) -> Value {
        self.context().data().user_data()
    }

    /// @yard
    /// Returns the amount of fuel in the {Store}.
    ///
    /// @return [Integer]
    /// @raise [Error] if fuel consumption is not enabled via {Wasmtime::Engine#new}
    pub fn get_fuel(&self) -> Result<u64, Error> {
        self.inner_ref().get_fuel().map_err(|e| error!("{}", e))
    }

    /// @yard
    /// Sets fuel to the {Store}.
    /// @param fuel [Integer] The new fuel amount.
    /// @def set_fuel(fuel)
    /// @raise [Error] if fuel consumption is not enabled via {Wasmtime::Engine#new}
    pub fn set_fuel(&self, fuel: u64) -> Result<(), Error> {
        unsafe { &mut *self.inner.get() }
            .set_fuel(fuel)
            .map_err(|e| error!("{}", e))?;

        Ok(())
    }

    /// @yard
    /// Sets the epoch deadline to a certain number of ticks in the future.
    ///
    /// Raises if there isn't enough fuel left in the {Store}, or
    /// when the {Engine}'s config does not have fuel enabled.
    ///
    /// @see ttps://docs.rs/wasmtime/latest/wasmtime/struct.Store.html#method.set_epoch_deadline Rust's doc on +set_epoch_deadline_ for more details.
    /// @def set_epoch_deadline(ticks_beyond_current)
    /// @param ticks_beyond_current [Integer] The number of ticks before this store reaches the deadline.
    /// @return [nil]
    pub fn set_epoch_deadline(&self, ticks_beyond_current: u64) {
        unsafe { &mut *self.inner.get() }.set_epoch_deadline(ticks_beyond_current);
    }

    /// @yard
    /// @def linear_memory_limit_hit?
    /// Returns whether the linear memory limit has been hit.
    ///
    /// @return [Boolean]
    pub fn linear_memory_limit_hit(&self) -> bool {
        self.context().data().store_limits.linear_memory_limit_hit()
    }

    /// @yard
    /// Returns the maximum linear memory consumed.
    ///
    /// @return [Integer]
    pub fn max_linear_memory_consumed(&self) -> usize {
        self.context()
            .data()
            .store_limits
            .max_linear_memory_consumed()
    }

    pub fn context(&self) -> StoreContext<'_, StoreData> {
        unsafe { (*self.inner.get()).as_context() }
    }

    pub fn context_mut(&self) -> StoreContextMut<'_, StoreData> {
        unsafe { (*self.inner.get()).as_context_mut() }
    }

    pub fn retain(&self, value: Value) {
        self.context_mut().data_mut().retain(value);
    }

    pub fn take_last_error(&self) -> Option<Error> {
        self.context_mut().data_mut().take_error()
    }

    fn inner_ref(&self) -> &StoreImpl<StoreData> {
        unsafe { &*self.inner.get() }
    }
}

/// A wrapper around a Ruby Value that has a store context.
/// Used in places where both Store or Caller can be used.
#[derive(Clone, Copy)]
pub enum StoreContextValue<'a> {
    Store(Opaque<Obj<Store>>),
    Caller(Opaque<Obj<Caller<'a>>>),
}

impl From<Obj<Store>> for StoreContextValue<'_> {
    fn from(store: Obj<Store>) -> Self {
        StoreContextValue::Store(store.into())
    }
}

impl<'a> From<Obj<Caller<'a>>> for StoreContextValue<'a> {
    fn from(caller: Obj<Caller<'a>>) -> Self {
        StoreContextValue::Caller(caller.into())
    }
}

impl StoreContextValue<'_> {
    pub fn mark(&self, marker: &Marker) {
        match self {
            Self::Store(store) => marker.mark(*store),
            Self::Caller(_) => {
                // The Caller is on the stack while it's "live". Right before the end of a host call,
                // we remove the Caller form the Ruby object, thus there is no need to mark.
            }
        }
    }

    pub fn context(&self) -> Result<StoreContext<'_, StoreData>, Error> {
        let ruby = Ruby::get().unwrap();
        match self {
            Self::Store(store) => Ok(ruby.get_inner_ref(store).context()),
            Self::Caller(caller) => ruby.get_inner_ref(caller).context(),
        }
    }

    pub fn context_mut(&self) -> Result<StoreContextMut<'_, StoreData>, Error> {
        let ruby = Ruby::get().unwrap();
        match self {
            Self::Store(store) => Ok(ruby.get_inner_ref(store).context_mut()),
            Self::Caller(caller) => ruby.get_inner_ref(caller).context_mut(),
        }
    }

    pub fn set_last_error(&self, error: Error) {
        let ruby = Ruby::get().unwrap();
        match self {
            Self::Store(store) => ruby
                .get_inner(*store)
                .context_mut()
                .data_mut()
                .set_error(error),
            Self::Caller(caller) => {
                if let Ok(mut context) = ruby.get_inner(*caller).context_mut() {
                    context.data_mut().set_error(error);
                }
            }
        };
    }

    pub fn handle_wasm_error(&self, error: wasmtime::Error) -> Error {
        if let Ok(Some(error)) = self.take_last_error() {
            error
        } else if let Some(exit) = error.downcast_ref::<I32Exit>() {
            wasi_exit_error().new_instance((exit.0,)).unwrap().into()
        } else {
            Trap::try_from(error)
                .map(|trap| trap.into())
                .unwrap_or_else(|e| error!("{}", e))
        }
    }

    pub fn retain(&self, value: Value) -> Result<(), Error> {
        self.context_mut()?.data_mut().retain(value);
        Ok(())
    }

    fn take_last_error(&self) -> Result<Option<Error>, Error> {
        let ruby = Ruby::get().unwrap();
        match self {
            Self::Store(store) => Ok(ruby.get_inner(*store).take_last_error()),
            Self::Caller(caller) => Ok(ruby
                .get_inner(*caller)
                .context_mut()?
                .data_mut()
                .take_error()),
        }
    }
}

fn hash_to_store_limits_builder(ruby: &Ruby, limits: RHash) -> Result<StoreLimitsBuilder, Error> {
    let mut limiter: StoreLimitsBuilder = StoreLimitsBuilder::new();

    if let Some(memory_size) = limits.lookup::<_, Option<usize>>(ruby.sym_new("memory_size"))? {
        limiter = limiter.memory_size(memory_size);
    }

    if let Some(table_elements) =
        limits.lookup::<_, Option<usize>>(ruby.sym_new("table_elements"))?
    {
        limiter = limiter.table_elements(table_elements);
    }

    if let Some(instances) = limits.lookup::<_, Option<u64>>(ruby.sym_new("instances"))? {
        limiter = limiter.instances(instances as usize);
    }

    if let Some(tables) = limits.lookup::<_, Option<u64>>(ruby.sym_new("tables"))? {
        limiter = limiter.tables(tables as usize);
    }

    if let Some(memories) = limits.lookup::<_, Option<u64>>(ruby.sym_new("memories"))? {
        limiter = limiter.memories(memories as usize);
    }

    Ok(limiter)
}

pub fn init(ruby: &Ruby) -> Result<(), Error> {
    let class = root().define_class("Store", ruby.class_object())?;

    class.define_singleton_method("new", function!(Store::new, -1))?;
    class.define_method("data", method!(Store::data, 0))?;
    class.define_method("get_fuel", method!(Store::get_fuel, 0))?;
    class.define_method("set_fuel", method!(Store::set_fuel, 1))?;
    class.define_method("set_epoch_deadline", method!(Store::set_epoch_deadline, 1))?;
    class.define_method(
        "linear_memory_limit_hit?",
        method!(Store::linear_memory_limit_hit, 0),
    )?;
    class.define_method(
        "max_linear_memory_consumed",
        method!(Store::max_linear_memory_consumed, 0),
    )?;

    Ok(())
}

/// A resource limiter proxy used to report memory usage to Ruby's GC.
struct TrackingResourceLimiter {
    inner: StoreLimits,
    tracker: ManuallyTracked<()>,
    linear_memory_limit_hit: bool,
    max_linear_memory_consumed: usize,
}

impl TrackingResourceLimiter {
    /// Create a new tracking limiter around an inner limiter.
    pub fn new(resource_limiter: StoreLimits) -> Self {
        Self {
            inner: resource_limiter,
            tracker: ManuallyTracked::new(0),
            linear_memory_limit_hit: false,
            max_linear_memory_consumed: 0,
        }
    }

    pub fn linear_memory_limit_hit(&self) -> bool {
        self.linear_memory_limit_hit
    }

    pub fn max_linear_memory_consumed(&self) -> usize {
        self.max_linear_memory_consumed
    }
}

impl ResourceLimiter for TrackingResourceLimiter {
    fn memory_growing(
        &mut self,
        current: usize,
        desired: usize,
        maximum: Option<usize>,
    ) -> wasmtime::Result<bool> {
        let res = self.inner.memory_growing(current, desired, maximum);

        // Update max_linear_memory_consumed
        self.max_linear_memory_consumed = self.max_linear_memory_consumed.max(desired);

        if res.is_ok() {
            self.tracker.increase_memory_usage(desired - current);
        } else {
            self.linear_memory_limit_hit = true;
        }

        if let Ok(allowed) = res {
            if !allowed {
                self.linear_memory_limit_hit = true;
            }
        }

        res
    }

    fn table_growing(
        &mut self,
        current: usize,
        desired: usize,
        maximum: Option<usize>,
    ) -> wasmtime::Result<bool> {
        self.inner.table_growing(current, desired, maximum)
    }

    fn memory_grow_failed(&mut self, error: wasmtime::Error) -> wasmtime::Result<()> {
        self.linear_memory_limit_hit = true;
        self.inner.memory_grow_failed(error)
    }

    fn table_grow_failed(&mut self, error: wasmtime::Error) -> wasmtime::Result<()> {
        self.inner.table_grow_failed(error)
    }

    fn instances(&self) -> usize {
        self.inner.instances()
    }

    fn tables(&self) -> usize {
        self.inner.tables()
    }

    fn memories(&self) -> usize {
        self.inner.memories()
    }
}

impl WasiView for StoreData {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        let ctx = self
            .wasi
            .as_mut()
            .expect("Should have WASI context defined if using WASI p2");
        let table = &mut self.resource_table;

        WasiCtxView { ctx, table }
    }
}
