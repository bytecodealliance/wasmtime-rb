use crate::ruby_api::{component::Func, Store};
use std::{borrow::BorrowMut, cell::RefCell};

use crate::error;
use magnus::{
    class,
    error::ErrorType,
    exception::arg_error,
    function,
    gc::Marker,
    method,
    prelude::*,
    r_string::RString,
    scan_args,
    typed_data::Obj,
    value::{self, ReprValue},
    DataTypeFunctions, Error, RArray, Ruby, TryConvert, TypedData, Value,
};
use magnus::{IntoValue, RModule};
use wasmtime::component::{Instance as InstanceImpl, Type, Val};

/// @yard
/// Represents a WebAssembly component instance.
/// @see https://docs.rs/wasmtime/latest/wasmtime/component/struct.Instance.html Wasmtime's Rust doc
#[derive(Clone, TypedData)]
#[magnus(class = "Wasmtime::Component::Instance", mark, free_immediately)]
pub struct Instance {
    inner: InstanceImpl,
    store: Obj<Store>,
}

unsafe impl Send for Instance {}

impl DataTypeFunctions for Instance {
    fn mark(&self, marker: &Marker) {
        marker.mark(self.store)
    }
}

impl Instance {
    pub fn from_inner(store: Obj<Store>, inner: InstanceImpl) -> Self {
        Self { inner, store }
    }

    /// @yard
    /// Retrieves a Wasm function from the component instance and calls it.
    ///
    /// @def invoke(name, *args)
    /// @param name [String] The name of function  to run.
    /// @param (see Component::Func#call)
    /// @return (see Component::Func#call)
    /// @see Component::Func#call
    pub fn invoke(&self, args: &[Value]) -> Result<Value, Error> {
        let name = RString::try_convert(*args.first().ok_or_else(|| {
            Error::new(
                magnus::exception::type_error(),
                "wrong number of arguments (given 0, expected 1+)",
            )
        })?)?;

        let func = self
            .inner
            .get_func(self.store.context_mut(), unsafe { name.as_str()? })
            .ok_or_else(|| error!("function \"{}\" not found", name))?;

        Func::invoke(&self.store.into(), &func, &args[1..])
    }
}

pub fn init(_ruby: &Ruby, namespace: &RModule) -> Result<(), Error> {
    let instance = namespace.define_class("Instance", class::object())?;
    instance.define_method("invoke", method!(Instance::invoke, -1))?;

    Ok(())
}
