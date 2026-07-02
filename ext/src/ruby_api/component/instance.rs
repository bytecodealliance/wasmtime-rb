use crate::ruby_api::{
    component::{ExportIndex, Func, ResourceType},
    Store,
};
use std::{borrow::BorrowMut, cell::RefCell};

use crate::error;
use magnus::{
    class,
    error::ErrorType,
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
use wasmtime::component::{ComponentExportIndex, Instance as InstanceImpl, Type, Val};

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
    /// Retrieves a Wasm function from the component instance.
    ///
    /// @def get_func(handle)
    /// @param handle [String, Array<String, ExportIndex>, ExportIndex] The path of the function to retrieve
    /// @return [Func, nil] The function if it exists, nil otherwise
    ///
    /// @example Retrieve a top-level +add+ export:
    ///   instance.get_func("add")
    ///
    /// @example Retrieve an +add+ export nested under an +adder+ instance top-level export:
    ///   instance.get_func(["adder", "add"])
    pub fn get_func(rb_self: Obj<Self>, handle: Value) -> Result<Option<Func>, Error> {
        let func = rb_self
            .export_index(handle)?
            .and_then(|index| rb_self.inner.get_func(rb_self.store.context_mut(), index))
            .map(|inner| Func::from_inner(inner, rb_self, rb_self.store));

        Ok(func)
    }

    /// @yard
    /// Retrieves a handle to a named export within the component instance,
    /// without resolving it to a concrete kind (function, resource, etc). The
    /// returned {ExportIndex} can be passed back as (or within) the +handle+
    /// argument to {#get_func}, {#get_resource}, or {#get_export_index}
    /// itself, to avoid re-resolving the same nested export by name
    /// repeatedly.
    ///
    /// @def get_export_index(handle)
    /// @param handle [String, Array<String, ExportIndex>, ExportIndex] The path of the export to retrieve
    /// @return [ExportIndex, nil] The export index if it exists, nil otherwise
    ///
    /// @example Retrieve the index of a nested +resource+ instance's export, then reuse it:
    ///   idx = instance.get_export_index("resource")
    ///   instance.get_func([idx, "[constructor]wrapped-string"])
    pub fn get_export_index(
        rb_self: Obj<Self>,
        handle: Value,
    ) -> Result<Option<ExportIndex>, Error> {
        let index = rb_self.export_index(handle)?.map(ExportIndex::from_inner);

        Ok(index)
    }

    /// @yard
    /// Retrieves an exported WIT +resource+ type from the component instance.
    ///
    /// @def get_resource(handle)
    /// @param handle [String, Array<String, ExportIndex>, ExportIndex] The path of the resource type to retrieve
    /// @return [ResourceType, nil] The resource type if it exists, nil otherwise
    ///
    /// @example Retrieve the +wrapped-string+ resource type nested under a +resource+ export:
    ///   instance.get_resource(["resource", "wrapped-string"])
    pub fn get_resource(rb_self: Obj<Self>, handle: Value) -> Result<Option<ResourceType>, Error> {
        let resource_type = rb_self
            .export_index(handle)?
            .and_then(|index| {
                rb_self
                    .inner
                    .get_resource(rb_self.store.context_mut(), index)
            })
            .map(ResourceType::from_inner);

        Ok(resource_type)
    }

    fn export_index(&self, handle: Value) -> Result<Option<ComponentExportIndex>, Error> {
        let ruby = Ruby::get_with(handle);
        let invalid_arg = || {
            Error::new(
                ruby.exception_type_error(),
                format!(
                    "invalid argument for component index, expected String | Array<String, ExportIndex> | ExportIndex, got {}",
                    handle.inspect()
                ),
            )
        };

        if let Some(name) = RString::from_value(handle) {
            return Ok(self
                .inner
                .get_export_index(self.store.context_mut(), None, unsafe { name.as_str()? }));
        }

        if let Some(elements) = RArray::from_value(handle) {
            let index = unsafe { elements.as_slice() }
                .iter()
                .try_fold::<_, _, Result<_, Error>>(None, |index, element| {
                    self.resolve_element(*element, index, invalid_arg)
                })?;

            return Ok(index);
        }

        if let Ok(export_index) = <&ExportIndex>::try_convert(handle) {
            return Ok(Some(export_index.inner()));
        }

        Err(invalid_arg())
    }

    fn resolve_element(
        &self,
        element: Value,
        index: Option<ComponentExportIndex>,
        invalid_arg: impl Fn() -> Error,
    ) -> Result<Option<ComponentExportIndex>, Error> {
        if let Some(name) = RString::from_value(element) {
            Ok(self
                .inner
                .get_export_index(self.store.context_mut(), index.as_ref(), unsafe {
                    name.as_str()?
                }))
        } else if let Ok(export_index) = <&ExportIndex>::try_convert(element) {
            Ok(Some(export_index.inner()))
        } else {
            Err(invalid_arg())
        }
    }
}

pub fn init(ruby: &Ruby, namespace: &RModule) -> Result<(), Error> {
    let instance = namespace.define_class("Instance", ruby.class_object())?;
    instance.define_method("get_func", method!(Instance::get_func, 1))?;
    instance.define_method("get_export_index", method!(Instance::get_export_index, 1))?;
    instance.define_method("get_resource", method!(Instance::get_resource, 1))?;

    Ok(())
}
