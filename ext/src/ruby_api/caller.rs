use super::{convert::WrapWasmtimeType, externals::Extern, root, store::StoreData};
use crate::{error, unsafe_impl_send_sync};
use magnus::{
    class,
    gc::{Compactor, Marker},
    method,
    prelude::*,
    typed_data::Obj,
    DataTypeFunctions, Error, RString, TypedData, Value,
};
use std::cell::UnsafeCell;
use wasmtime::{AsContext, AsContextMut, Caller as CallerImpl, StoreContext, StoreContextMut};

/// A handle to a [`wasmtime::Caller`] that's only valid during a Func execution.
/// [`UnsafeCell`] wraps the wasmtime::Caller because the Value's lifetime can't
/// be tied to the Caller: the Value is handed back to Ruby and we can't control
/// whether the user keeps a handle to it or not.
#[derive(Debug)]
pub struct CallerHandle<'a> {
    caller: UnsafeCell<Option<CallerImpl<'a, StoreData>>>,
}

impl<'a> CallerHandle<'a> {
    pub fn new(caller: CallerImpl<'a, StoreData>) -> Self {
        Self {
            caller: UnsafeCell::new(Some(caller)),
        }
    }

    pub fn get_mut(&self) -> Result<&mut CallerImpl<'a, StoreData>, Error> {
        unsafe { &mut *self.caller.get() }
            .as_mut()
            .ok_or_else(|| error!("Caller outlived its Func execution"))
    }

    pub fn get(&self) -> Result<&CallerImpl<'a, StoreData>, Error> {
        unsafe { (*self.caller.get()).as_ref() }
            .ok_or_else(|| error!("Caller outlived its Func execution"))
    }

    pub fn expire(&self) {
        unsafe { *self.caller.get() = None }
    }
}

/// @yard
/// @rename Wasmtime::Caller
/// Represents the Caller's context within a Func execution. An instance of
/// Caller is sent as the first parameter to Func's implementation (the
/// block argument in {Func.new}).
/// @see https://docs.rs/wasmtime/latest/wasmtime/struct.Caller.html Wasmtime's Rust doc
#[derive(Debug, TypedData)]
#[magnus(
    class = "Wasmtime::Caller",
    free_immediately,
    unsafe_generics,
    mark,
    size
)]
pub struct Caller<'a> {
    handle: CallerHandle<'a>,
}

impl DataTypeFunctions for Caller<'_> {
    fn mark(&self, marker: &Marker) {
        self.handle
            .get()
            .map(|c| c.data().mark(marker))
            .unwrap_or_default()
    }

    fn compact(&self, compactor: &Compactor) {
        self.handle
            .get_mut()
            .map(|c| c.data_mut().compact(compactor))
            .unwrap_or_default()
    }
}

impl<'a> Caller<'a> {
    pub fn new(caller: CallerImpl<'a, StoreData>) -> Self {
        Self {
            handle: CallerHandle::new(caller),
        }
    }

    /// @yard
    /// Returns the store's data. Akin to {Store#data}.
    /// @return [Object] The store's data (the object passed to {Store.new}).
    pub fn store_data(&self) -> Result<Value, Error> {
        self.context().map(|ctx| ctx.data().user_data())
    }

    /// @yard
    /// @def export(name)
    /// @see Instance#export
    pub fn export(rb_self: Obj<Caller<'a>>, name: RString) -> Result<Option<Extern<'a>>, Error> {
        let caller = rb_self;
        let inner = caller.handle.get_mut()?;

        if let Some(export) = inner.get_export(unsafe { name.as_str() }?) {
            export.wrap_wasmtime_type(rb_self.into()).map(Some)
        } else {
            Ok(None)
        }
    }

    /// @yard
    /// (see Store#fuel_consumed)
    pub fn fuel_consumed(&self) -> Result<Option<u64>, Error> {
        self.handle.get().map(|c| c.fuel_consumed())
    }

    /// @yard
    /// (see Store#add_fuel)
    /// @def add_fuel(fuel)
    pub fn add_fuel(&self, fuel: u64) -> Result<(), Error> {
        self.handle
            .get_mut()
            .and_then(|c| c.add_fuel(fuel).map_err(|e| error!("{}", e)))?;

        Ok(())
    }

    /// @yard
    /// (see Store#consume_fuel)
    /// @def consume_fuel(fuel)
    pub fn consume_fuel(&self, fuel: u64) -> Result<u64, Error> {
        self.handle
            .get_mut()
            .and_then(|c| c.consume_fuel(fuel).map_err(|e| error!("{}", e)))
    }

    pub fn context(&self) -> Result<StoreContext<StoreData>, Error> {
        self.handle.get().map(|c| c.as_context())
    }

    pub fn context_mut(&self) -> Result<StoreContextMut<StoreData>, Error> {
        self.handle.get_mut().map(|c| c.as_context_mut())
    }

    pub fn expire(&self) {
        self.handle.expire();
    }
}

unsafe_impl_send_sync!(Caller);

pub fn init() -> Result<(), Error> {
    let klass = root().define_class("Caller", class::object())?;
    klass.define_method("store_data", method!(Caller::store_data, 0))?;
    klass.define_method("export", method!(Caller::export, 1))?;
    klass.define_method("fuel_consumed", method!(Caller::fuel_consumed, 0))?;
    klass.define_method("add_fuel", method!(Caller::add_fuel, 1))?;
    klass.define_method("consume_fuel", method!(Caller::consume_fuel, 1))?;

    Ok(())
}
