use super::{convert::WrapWasmtimeType, externals::Extern, root, store::StoreData};
use crate::error;
use magnus::{class, method, typed_data::Obj, Error, Module as _, RString, Ruby, Value};
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

    // Note that the underlying implemenation relies on `UnsafeCell`, which
    // provides some gurantees around interior mutability, therefore we're
    // opting to allow this lint.
    #[allow(clippy::mut_from_ref)]
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
#[derive(Debug)]
#[magnus::wrap(class = "Wasmtime::Caller", free_immediately, unsafe_generics)]
pub struct Caller<'a> {
    handle: CallerHandle<'a>,
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
        let inner = rb_self.handle.get_mut()?;

        if let Some(export) = inner.get_export(unsafe { name.as_str() }?) {
            export.wrap_wasmtime_type(rb_self.into()).map(Some)
        } else {
            Ok(None)
        }
    }

    /// @yard
    /// (see Store#get_fuel)
    /// @def get_fuel
    pub fn get_fuel(&self) -> Result<u64, Error> {
        self.handle
            .get()
            .map(|c| c.get_fuel())?
            .map_err(|e| error!("{}", e))
    }

    /// @yard
    /// (see Store#set_fuel)
    /// @def set_fuel(fuel)
    pub fn set_fuel(&self, fuel: u64) -> Result<(), Error> {
        self.handle
            .get_mut()
            .and_then(|c| c.set_fuel(fuel).map_err(|e| error!("{}", e)))?;

        Ok(())
    }

    pub fn context(&self) -> Result<StoreContext<'_, StoreData>, Error> {
        self.handle.get().map(|c| c.as_context())
    }

    pub fn context_mut(&self) -> Result<StoreContextMut<'_, StoreData>, Error> {
        self.handle.get_mut().map(|c| c.as_context_mut())
    }

    pub fn expire(&self) {
        self.handle.expire();
    }
}

unsafe impl Send for Caller<'_> {}

pub fn init(ruby: &Ruby) -> Result<(), Error> {
    let klass = root().define_class("Caller", ruby.class_object())?;
    klass.define_method("store_data", method!(Caller::store_data, 0))?;
    klass.define_method("export", method!(Caller::export, 1))?;
    klass.define_method("get_fuel", method!(Caller::get_fuel, 0))?;
    klass.define_method("set_fuel", method!(Caller::set_fuel, 1))?;

    Ok(())
}
