use super::errors::wasi_exit_error;
use super::{caller::Caller, engine::Engine, root, trap::Trap, wasi_ctx_builder::WasiCtxBuilder};
use crate::{define_rb_intern, error, helpers::WrappedStruct};
use magnus::Class;
use magnus::{
    exception::Exception, function, gc, method, scan_args, value::BoxValue, DataTypeFunctions,
    Error, Module, Object, TypedData, Value, QNIL,
};
use std::cell::UnsafeCell;
use std::convert::TryFrom;
use wasmtime::{AsContext, AsContextMut, Store as StoreImpl, StoreContext, StoreContextMut};
use wasmtime_wasi::{I32Exit, WasiCtx};

define_rb_intern!(
    WASI_CTX => "wasi_ctx",
);

pub struct StoreData {
    user_data: Value,
    host_exception: HostException,
    wasi: Option<WasiCtx>,
    refs: Vec<Value>,
}

type BoxedException = BoxValue<Exception>;
#[derive(Debug, Default)]
pub struct HostException(Option<BoxedException>);
impl HostException {
    pub fn take(&mut self) -> Option<Exception> {
        std::mem::take(&mut self.0).map(|e| e.to_owned())
    }

    pub fn hold(&mut self, e: Exception) {
        self.0 = Some(BoxValue::new(e));
    }
}

impl StoreData {
    pub fn exception(&mut self) -> &mut HostException {
        &mut self.host_exception
    }

    pub fn take_last_error(&mut self) -> Option<Error> {
        self.host_exception.take().map(Error::from)
    }

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

    pub fn mark(&self) {
        gc::mark(&self.user_data);
        self.refs.iter().for_each(gc::mark);
    }
}

/// @yard
/// Represents a WebAssebmly store.
/// @see https://docs.rs/wasmtime/latest/wasmtime/struct.Store.html Wasmtime's Rust doc
#[derive(Debug, TypedData)]
#[magnus(class = "Wasmtime::Store", size, mark, free_immediatly)]
pub struct Store {
    inner: UnsafeCell<StoreImpl<StoreData>>,
}

impl DataTypeFunctions for Store {
    fn mark(&self) {
        self.context().data().mark();
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
    pub fn new(args: &[Value]) -> Result<Self, Error> {
        let args = scan_args::scan_args::<(&Engine,), (Option<Value>,), (), (), _, ()>(args)?;
        let kw = scan_args::get_kwargs::<_, (), (Option<&WasiCtxBuilder>,), ()>(
            args.keywords,
            &[],
            &[*WASI_CTX],
        )?;
        let (engine,) = args.required;
        let (user_data,) = args.optional;
        let user_data = user_data.unwrap_or_else(|| QNIL.into());
        let wasi = match kw.optional.0 {
            None => None,
            Some(wasi_ctx_builder) => Some(wasi_ctx_builder.build_context()?),
        };

        let eng = engine.get();
        let store_data = StoreData {
            user_data,
            host_exception: HostException::default(),
            wasi,
            refs: Default::default(),
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
    /// Returns the amount of fuel consumed by this {Store}’s execution so far,
    /// or +nil+ when the {Engine}’s {Config} does not have fuel enabled.
    /// @return [Integer, Nil]
    pub fn fuel_consumed(&self) -> Option<u64> {
        self.inner_ref().fuel_consumed()
    }

    /// @yard
    /// Adds fuel to the {Store}.
    /// @param fuel [Integer] The fuel to add.
    /// @def add_fuel(fuel)
    /// @return [Nil]
    pub fn add_fuel(&self, fuel: u64) -> Result<Value, Error> {
        unsafe { &mut *self.inner.get() }
            .add_fuel(fuel)
            .map_err(|e| error!("{}", e))?;

        Ok(*QNIL)
    }

    /// @yard
    /// Synthetically consumes fuel from this {Store}.
    /// Raises if there isn't enough fuel left in the {Store}, or
    /// when the {Engine}’s {Config} does not have fuel enabled.
    ///
    /// @param fuel [Integer] The fuel to consume.
    /// @def consume_fuel(fuel)
    /// @return [Integer] The remaining fuel.
    pub fn consume_fuel(&self, fuel: u64) -> Result<u64, Error> {
        unsafe { &mut *self.inner.get() }
            .consume_fuel(fuel)
            .map_err(|e| error!("{}", e))
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

    fn inner_ref(&self) -> &StoreImpl<StoreData> {
        unsafe { &*self.inner.get() }
    }
}

/// A wrapper around a Ruby Value that has a store context.
/// Used in places where both Store or Caller can be used.
#[derive(Debug, Clone, Copy)]
pub enum StoreContextValue<'a> {
    Store(WrappedStruct<Store>),
    Caller(WrappedStruct<Caller<'a>>),
}

impl<'a> From<WrappedStruct<Store>> for StoreContextValue<'a> {
    fn from(store: WrappedStruct<Store>) -> Self {
        StoreContextValue::Store(store)
    }
}

impl<'a> From<WrappedStruct<Caller<'a>>> for StoreContextValue<'a> {
    fn from(caller: WrappedStruct<Caller<'a>>) -> Self {
        StoreContextValue::Caller(caller)
    }
}

impl<'a> StoreContextValue<'a> {
    pub fn mark(&self) {
        match self {
            Self::Store(store) => store.mark(),
            Self::Caller(_) => {
                // The Caller is on the stack while it's "live". Right before the end of a host call,
                // we remove the Caller form the Ruby object, thus there is no need to mark.
            }
        }
    }

    pub fn context(&self) -> Result<StoreContext<StoreData>, Error> {
        match self {
            Self::Store(store) => Ok(store.get()?.context()),
            Self::Caller(caller) => caller.get()?.context(),
        }
    }

    pub fn context_mut(&self) -> Result<StoreContextMut<StoreData>, Error> {
        match self {
            Self::Store(store) => Ok(store.get()?.context_mut()),
            Self::Caller(caller) => caller.get()?.context_mut(),
        }
    }

    pub fn handle_wasm_error(&self, error: anyhow::Error) -> Error {
        match self.context_mut() {
            Ok(mut context) => context.data_mut().take_last_error().unwrap_or_else(|| {
                if let Some(exit) = error.downcast_ref::<I32Exit>() {
                    wasi_exit_error().new_instance((exit.0,)).unwrap().into()
                } else {
                    Trap::try_from(error)
                        .map(|trap| trap.into())
                        .unwrap_or_else(|e| error!("{}", e))
                }
            }),
            Err(e) => e,
        }
    }

    pub fn retain(&self, value: Value) -> Result<(), Error> {
        self.context_mut()?.data_mut().retain(value);
        Ok(())
    }
}

pub fn init() -> Result<(), Error> {
    let class = root().define_class("Store", Default::default())?;

    class.define_singleton_method("new", function!(Store::new, -1))?;
    class.define_method("data", method!(Store::data, 0))?;
    class.define_method("fuel_consumed", method!(Store::fuel_consumed, 0))?;
    class.define_method("add_fuel", method!(Store::add_fuel, 1))?;
    class.define_method("consume_fuel", method!(Store::consume_fuel, 1))?;

    Ok(())
}
