use super::errors::wasi_exit_error;
use super::{caller::Caller, engine::Engine, root, trap::Trap, wasi_ctx_builder::WasiCtxBuilder};
use crate::{define_rb_intern, error};
use magnus::rb_sys::AsRawValue;
use magnus::Class;
use magnus::{
    class, function,
    gc::{Compactor, Marker},
    method, scan_args,
    typed_data::Obj,
    value::Opaque,
    DataTypeFunctions, Error, IntoValue, Module, Object, Ruby, TypedData, Value,
};
use rb_sys::rb_gc_guard;
use std::cell::UnsafeCell;
use std::convert::TryFrom;
use wasmtime::{AsContext, AsContextMut, Store as StoreImpl, StoreContext, StoreContextMut};
use wasmtime_wasi::{I32Exit, WasiCtx};

define_rb_intern!(
    WASI_CTX => "wasi_ctx",
);

pub struct StoreData {
    user_data: Value,
    wasi: Option<WasiCtx>,
    refs: Vec<Value>,
    last_error: Option<Error>,
}

impl StoreData {
    pub fn user_data(&self) -> Value {
        self.user_data
    }

    pub fn has_wasi_ctx(&self) -> bool {
        self.wasi.is_some()
    }

    pub fn wasi_ctx_mut(&mut self) -> &mut WasiCtx {
        self.wasi.as_mut().expect("Store must have a WASI context")
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
            let _guarded = rb_gc_guard!(value.as_raw());
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
    /// @def new(engine, data = nil, wasi_ctx: nil)
    /// @param engine [Wasmtime::Engine]
    ///   The engine for this store.
    /// @param data [Object]
    ///   The data attached to the store. Can be retrieved through {Wasmtime::Store#data} and {Wasmtime::Caller#data}.
    /// @param wasi_ctx [Wasmtime::WasiCtxBuilder]
    ///   The WASI context to use in this store.
    /// @return [Wasmtime::Store]
    ///
    /// @example
    ///   store = Wasmtime::Store.new(Wasmtime::Engine.new)
    ///
    /// @example
    ///   store = Wasmtime::Store.new(Wasmtime::Engine.new, {})
    pub fn new(ruby: &Ruby, args: &[Value]) -> Result<Self, Error> {
        let args = scan_args::scan_args::<(Obj<Engine>,), (Option<Value>,), (), (), _, ()>(args)?;
        let kw = scan_args::get_kwargs::<_, (), (Option<&WasiCtxBuilder>,), ()>(
            args.keywords,
            &[],
            &[*WASI_CTX],
        )?;
        let (engine,) = args.required;
        let _ = rb_gc_guard!(engine.as_raw());
        let (user_data,) = args.optional;
        let user_data = user_data.unwrap_or_else(|| ruby.qnil().into_value());
        let _ = rb_gc_guard!(user_data.as_raw());
        let wasi = match kw.optional.0 {
            None => None,
            Some(wasi_ctx_builder) => Some(wasi_ctx_builder.build_context(ruby)?),
        };

        let store_data = StoreData {
            user_data,
            wasi,
            refs: Default::default(),
            last_error: Default::default(),
        };
        let store = Self {
            inner: UnsafeCell::new(StoreImpl::new((*engine).get(), store_data)),
        };

        Ok(store)
    }

    /// @yard
    /// @return [Object] The passed in value in {.new}
    pub fn data(&self) -> Value {
        self.context().data().user_data()
    }

    /// @yard
    /// Returns the amount of fuel consumed by this {Store}’s execution so far,
    /// or +nil+ when the {Engine}’s config does not have fuel enabled.
    /// @return [Integer, Nil]
    pub fn fuel_consumed(&self) -> Option<u64> {
        self.inner_store().fuel_consumed()
    }

    /// @yard
    /// Adds fuel to the {Store}.
    /// @param fuel [Integer] The fuel to add.
    /// @def add_fuel(fuel)
    /// @return [Nil]
    pub fn add_fuel(&self, fuel: u64) -> Result<(), Error> {
        self.inner_store_mut()
            .add_fuel(fuel)
            .map_err(|e| error!("{}", e))?;

        Ok(())
    }

    /// @yard
    /// Synthetically consumes fuel from this {Store}.
    /// Raises if there isn't enough fuel left in the {Store}, or
    /// when the {Engine}’s config does not have fuel enabled.
    ///
    /// @param fuel [Integer] The fuel to consume.
    /// @def consume_fuel(fuel)
    /// @return [Integer] The remaining fuel.
    pub fn consume_fuel(&self, fuel: u64) -> Result<u64, Error> {
        self.inner_store_mut()
            .consume_fuel(fuel)
            .map_err(|e| error!("{}", e))
    }

    /// @yard
    /// Sets the epoch deadline to a certain number of ticks in the future.
    ///
    /// Raises if there isn't enough fuel left in the {Store}, or
    /// when the {Engine}’s config does not have fuel enabled.
    ///
    /// @see ttps://docs.rs/wasmtime/latest/wasmtime/struct.Store.html#method.set_epoch_deadline Rust's doc on +set_epoch_deadline_ for more details.
    /// @def set_epoch_deadline(ticks_beyond_current)
    /// @param ticks_beyond_current [Integer] The number of ticks before this store reaches the deadline.
    /// @return [nil]
    pub fn set_epoch_deadline(&self, ticks_beyond_current: u64) {
        self.inner_store_mut()
            .set_epoch_deadline(ticks_beyond_current);
    }

    pub fn context(&self) -> StoreContext<StoreData> {
        self.inner_store().as_context()
    }

    pub fn context_mut(&self) -> StoreContextMut<StoreData> {
        self.inner_store_mut().as_context_mut()
    }

    pub fn retain(&self, value: Value) {
        self.context_mut().data_mut().retain(value);
    }

    pub fn take_last_error(&self) -> Option<Error> {
        self.context_mut().data_mut().take_error()
    }

    fn inner_store(&self) -> &StoreImpl<StoreData> {
        let _ = Ruby::get().expect("must be called from a Ruby thread holding the GVL");
        // SAFETY: Store is not Send, and we rely on the GVL to ensure no concurrent access.
        unsafe { &*self.inner.get() }
    }

    #[allow(clippy::mut_from_ref)]
    fn inner_store_mut(&self) -> &mut StoreImpl<StoreData> {
        let _ = Ruby::get().expect("must be called from a Ruby thread holding the GVL");
        // SAFETY: Store is not Send, and we rely on the GVL to ensure no concurrent access.
        unsafe { &mut *self.inner.get() }
    }
}

/// A wrapper around a Ruby Value that has a store context.
/// Used in places where both Store or Caller can be used.
#[derive(Clone, Copy)]
pub enum StoreContextValue<'a> {
    Store(Opaque<Obj<Store>>),
    Caller(Opaque<Obj<Caller<'a>>>),
}

impl<'a> From<Obj<Store>> for StoreContextValue<'a> {
    fn from(store: Obj<Store>) -> Self {
        StoreContextValue::Store(store.into())
    }
}

impl<'a> From<Obj<Caller<'a>>> for StoreContextValue<'a> {
    fn from(caller: Obj<Caller<'a>>) -> Self {
        StoreContextValue::Caller(caller.into())
    }
}

impl<'a> StoreContextValue<'a> {
    pub fn mark(&self, marker: &Marker) {
        match self {
            Self::Store(store) => marker.mark(*store),
            Self::Caller(caller) => marker.mark(*caller),
        }
    }

    pub fn context(&self) -> Result<StoreContext<StoreData>, Error> {
        let ruby = Ruby::get().expect("must be called from a Ruby thread holding the GVL");
        match self {
            Self::Store(store) => Ok(ruby.get_inner_ref(store).context()),
            Self::Caller(caller) => ruby.get_inner_ref(caller).context(),
        }
    }

    pub fn context_mut(&self) -> Result<StoreContextMut<StoreData>, Error> {
        let ruby = Ruby::get().expect("must be called from a Ruby thread holding the GVL");
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

    pub fn handle_wasm_error(&self, error: anyhow::Error) -> Error {
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

pub fn init() -> Result<(), Error> {
    let class = root().define_class("Store", class::object())?;

    class.define_singleton_method("new", function!(Store::new, -1))?;
    class.define_method("data", method!(Store::data, 0))?;
    class.define_method("fuel_consumed", method!(Store::fuel_consumed, 0))?;
    class.define_method("add_fuel", method!(Store::add_fuel, 1))?;
    class.define_method("consume_fuel", method!(Store::consume_fuel, 1))?;
    class.define_method("set_epoch_deadline", method!(Store::set_epoch_deadline, 1))?;

    Ok(())
}
