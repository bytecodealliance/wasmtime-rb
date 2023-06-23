use super::errors::wasi_exit_error;
use super::wasi_deterministic_ctx_builder;
use super::{caller::Caller, engine::Engine, root, trap::Trap, wasi_ctx_builder::WasiCtxBuilder, wasi_deterministic_ctx_builder::WasiDeterministicCtxBuilder};
use crate::{define_rb_intern, error};
use magnus::Class;
use magnus::{
    class, function,
    gc::{Compactor, Marker},
    method, scan_args,
    typed_data::Obj,
    value::Opaque,
    DataTypeFunctions, Error, IntoValue, Module, Object, Ruby, TypedData, Value,
};
use std::cell::UnsafeCell;
use std::convert::TryFrom;
use wasmtime::{AsContext, AsContextMut, Store as StoreImpl, StoreContext, StoreContextMut};
use wasmtime_wasi::{I32Exit, WasiCtx};

define_rb_intern!(
    WASI_CTX => "wasi_ctx",
    WASI_DET_CTX => "wasi_det_ctx",
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
        let args = scan_args::scan_args::<(&Engine,), (Option<Value>,), (), (), _, ()>(args)?;
        let kw = scan_args::get_kwargs::<_, (), (Option<&WasiCtxBuilder>, Option<&WasiDeterministicCtxBuilder>), ()>(
            args.keywords,
            &[],
            &[*WASI_CTX, *WASI_DET_CTX],
        )?;
        let (engine,) = args.required;
        let (user_data,) = args.optional;
        let user_data = user_data.unwrap_or_else(|| ().into_value());


        let (ctx, det) = kw.optional;
        let wasi = match (ctx, det) {
            (Some(_), Some(_)) => None,
            (Some(ctx), None) => Some(ctx.build_context(ruby)?),
            (None, Some(det)) => Some(det.build_context(ruby)?),
            (None, None) => None,
        };


        let eng = engine.get();
        let store_data = StoreData {
            user_data,
            wasi,
            refs: Default::default(),
            last_error: Default::default(),
        };
        let store = Self {
            inner: UnsafeCell::new(StoreImpl::new(eng, store_data)),
        };

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
    /// when the {Engine}’s config does not have fuel enabled.
    ///
    /// @see ttps://docs.rs/wasmtime/latest/wasmtime/struct.Store.html#method.set_epoch_deadline Rust's doc on +set_epoch_deadline_ for more details.
    /// @def set_epoch_deadline(ticks_beyond_current)
    /// @param ticks_beyond_current [Integer] The number of ticks before this store reaches the deadline.
    /// @return [nil]
    pub fn set_epoch_deadline(&self, ticks_beyond_current: u64) {
        unsafe { &mut *self.inner.get() }.set_epoch_deadline(ticks_beyond_current);
    }

    pub fn has_wasi_ctx(&self) -> bool {
        self.context().data().has_wasi_ctx()
    }

    pub fn context(&self) -> StoreContext<StoreData> {
        unsafe { (*self.inner.get()).as_context() }
    }

    pub fn context_mut(&self) -> StoreContextMut<StoreData> {
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
            Self::Caller(_) => {
                // The Caller is on the stack while it's "live". Right before the end of a host call,
                // we remove the Caller form the Ruby object, thus there is no need to mark.
            }
        }
    }

    pub fn context(&self) -> Result<StoreContext<StoreData>, Error> {
        let ruby = Ruby::get().unwrap();
        match self {
            Self::Store(store) => Ok(ruby.get_inner_ref(store).context()),
            Self::Caller(caller) => ruby.get_inner_ref(caller).context(),
        }
    }

    pub fn context_mut(&self) -> Result<StoreContextMut<StoreData>, Error> {
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
    class.define_method("get_fuel", method!(Store::get_fuel, 0))?;
    class.define_method("set_fuel", method!(Store::set_fuel, 1))?;
    class.define_method("set_epoch_deadline", method!(Store::set_epoch_deadline, 1))?;
    class.define_method("has_wasi_ctx", method!(Store::has_wasi_ctx, 0))?;

    Ok(())
}
