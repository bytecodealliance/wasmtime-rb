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
}

pub fn init(_ruby: &Ruby, namespace: &RModule) -> Result<(), Error> {
    let instance = namespace.define_class("Instance", class::object())?;

    Ok(())
}
