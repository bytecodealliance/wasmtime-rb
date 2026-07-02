use crate::ruby_api::{component::Func, Store};
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
    /// @param handle [String, Array<String>] The name of the export to
    ///   retrieve, or an +Array+ of names forming a path through nested
    ///   exports (each name resolved within the export found by the previous
    ///   one).
    ///
    ///   Each name must match the export's name in the component's WIT world
    ///   exactly:
    ///   - a plain (kebab-case) export name, e.g. +"add"+ for
    ///     +export add: func(...)+;
    ///   - a fully-qualified interface name, including namespace, package and
    ///     any version, e.g. +"my-namespace:my-package/my-interface@0.1.0"+
    ///     for +export my-interface+ — the interface's own members are then
    ///     addressed by the next name in the path;
    ///   - for functions attached to a WIT +resource+, the name is mangled in
    ///     the canonical ABI format: +"[constructor]my-resource"+,
    ///     +"[method]my-resource.my-func"+ or +"[static]my-resource.my-func"+.
    /// @return [Func, nil] The function if it exists, nil otherwise
    ///
    /// @example Retrieve a top-level +add+ export:
    ///   instance.get_func("add")
    ///
    /// @example Retrieve an +add+ export nested under an +adder+ instance top-level export:
    ///   instance.get_func(["adder", "add"])
    ///
    /// @example Retrieve the constructor of a +counter+ resource exported by a versioned interface:
    ///   instance.get_func(["my-namespace:my-package/types@0.1.0", "[constructor]counter"])
    pub fn get_func(rb_self: Obj<Self>, handle: Value) -> Result<Option<Func>, Error> {
        let func = rb_self
            .export_index(handle)?
            .and_then(|index| rb_self.inner.get_func(rb_self.store.context_mut(), index))
            .map(|inner| Func::from_inner(inner, rb_self, rb_self.store));

        Ok(func)
    }

    fn export_index(&self, handle: Value) -> Result<Option<ComponentExportIndex>, Error> {
        let ruby = Ruby::get_with(handle);
        let invalid_arg = || {
            Error::new(
                ruby.exception_type_error(),
                format!(
                    "invalid argument for component index, expected String | Array<String>, got {}",
                    handle.inspect()
                ),
            )
        };

        let index = if let Some(name) = RString::from_value(handle) {
            self.inner
                .get_export(self.store.context_mut(), None, unsafe { name.as_str()? })
                .map(|(_, index)| index)
        } else if let Some(names) = RArray::from_value(handle) {
            unsafe { names.as_slice() }
                .iter()
                .try_fold::<_, _, Result<_, Error>>(None, |index, name| {
                    let name = RString::from_value(*name).ok_or_else(invalid_arg)?;

                    Ok(self
                        .inner
                        .get_export(self.store.context_mut(), index.as_ref(), unsafe {
                            name.as_str()?
                        })
                        .map(|(_, index)| index))
                })?
        } else {
            return Err(invalid_arg());
        };

        Ok(index)
    }
}

pub fn init(ruby: &Ruby, namespace: &RModule) -> Result<(), Error> {
    let instance = namespace.define_class("Instance", ruby.class_object())?;
    instance.define_method("get_func", method!(Instance::get_func, 1))?;

    Ok(())
}
