use super::convert;
use super::types::{self, ComponentType};
use super::{Component, Instance};
use crate::{
    err,
    ruby_api::{
        errors,
        store::{StoreContextValue, StoreData},
        Engine, Module, Store,
    },
};
use std::{
    borrow::BorrowMut,
    cell::{RefCell, RefMut},
    collections::HashMap,
};

use crate::error;
use magnus::{
    block::Proc,
    class, function,
    gc::Marker,
    method,
    r_string::RString,
    scan_args::scan_args,
    typed_data::Obj,
    value::{Opaque, ReprValue},
    DataTypeFunctions, Error, Module as _, Object, RArray, RModule, Ruby, TryConvert, TypedData,
    Value,
};
use wasmtime::component::{Linker as LinkerImpl, LinkerInstance as LinkerInstanceImpl, Val};
use wasmtime_wasi::{ResourceTable, WasiCtx};

/// Stores type information for a registered host function
#[derive(Clone, Debug)]
struct FuncTypeInfo {
    #[allow(dead_code)]
    path: String, // e.g., "" for root, "math" for nested instance
    param_types: Vec<ComponentType>,
    result_types: Vec<ComponentType>,
}

/// @yard
/// @rename Wasmtime::Component::Linker
/// @see https://docs.rs/wasmtime/latest/wasmtime/component/struct.Linker.html Wasmtime's Rust doc
#[derive(TypedData)]
#[magnus(class = "Wasmtime::Component::Linker", size, mark, free_immediately)]
pub struct Linker {
    inner: RefCell<LinkerImpl<StoreData>>,
    refs: RefCell<Vec<Value>>,
    has_wasi: RefCell<bool>,
    func_types: RefCell<HashMap<String, FuncTypeInfo>>,
}
unsafe impl Send for Linker {}

impl DataTypeFunctions for Linker {
    fn mark(&self, marker: &magnus::gc::Marker) {
        marker.mark_slice(self.refs.borrow().as_slice());
    }
}

impl Linker {
    /// @yard
    /// @def new(engine)
    /// @param engine [Engine]
    /// @return [Linker]
    pub fn new(engine: &Engine) -> Result<Self, Error> {
        let linker: LinkerImpl<StoreData> = LinkerImpl::new(engine.get());

        Ok(Linker {
            inner: RefCell::new(linker),
            refs: RefCell::new(Vec::new()),
            has_wasi: RefCell::new(false),
            func_types: RefCell::new(HashMap::new()),
        })
    }

    pub(crate) fn inner_mut(&self) -> RefMut<'_, LinkerImpl<StoreData>> {
        self.inner.borrow_mut()
    }

    pub(crate) fn has_wasi(&self) -> bool {
        *self.has_wasi.borrow()
    }

    /// @yard
    /// @def root
    /// Define items in the root of this {Linker}.
    /// @yield [instance] The block allows configuring the {LinkerInstance};
    ///   outside of this scope the instance becomes unusable.
    /// @yieldparam instance [LinkerInstance]
    /// @return [Linker] +self+
    pub fn root(ruby: &Ruby, rb_self: Obj<Self>) -> Result<Obj<Self>, Error> {
        let Ok(mut inner) = rb_self.inner.try_borrow_mut() else {
            return err!("Linker is not reentrant");
        };
        let instance = ruby.obj_wrap(LinkerInstance::from_inner(
            inner.root(),
            String::new(), // root path is empty
            rb_self.as_value(),
        ));
        let block_result: Result<Value, _> = ruby.yield_value(instance);

        instance.take_inner();

        match block_result {
            Ok(_) => Ok(rb_self),
            Err(e) => Err(e),
        }
    }

    /// @yard
    /// @def instance(name)
    /// Define items at the provided namespace in this {Linker}.
    /// @param name [String]
    /// @yield [instance] The block allows configuring the {LinkerInstance};
    ///   outside of this scope the instance becomes unusable.
    /// @yieldparam instance [LinkerInstance]
    /// @return [Linker] +self+
    pub fn instance(ruby: &Ruby, rb_self: Obj<Self>, name: RString) -> Result<Obj<Self>, Error> {
        let mut inner = rb_self.inner.borrow_mut();
        let name_str = unsafe { name.as_str() }?;
        let instance = inner.instance(name_str).map_err(|e| error!("{}", e))?;

        let instance = ruby.obj_wrap(LinkerInstance::from_inner(
            instance,
            name_str.to_string(),
            rb_self.as_value(),
        ));

        let block_result: Result<Value, _> = ruby.yield_value(instance);

        instance.take_inner();

        match block_result {
            Ok(_) => Ok(rb_self),
            Err(e) => Err(e),
        }
    }

    /// @yard
    /// Instantiates a {Component} in a {Store} using the defined imports in the linker.
    /// @def instantiate(store, component)
    /// @param store [Store]
    /// @param component [Component]
    /// @return [Instance]
    fn instantiate(
        _ruby: &Ruby,
        rb_self: Obj<Self>,
        store: Obj<Store>,
        component: &Component,
    ) -> Result<Instance, Error> {
        if *rb_self.has_wasi.borrow() && !store.context().data().has_wasi_ctx() {
            return err!("{}", errors::missing_wasi_ctx_error("linker.instantiate"));
        }

        Self::validate_host_function_signatures(&rb_self, component)?;

        let inner = rb_self.inner.borrow();
        inner
            .instantiate(store.context_mut(), component.get())
            .map(|instance| {
                rb_self
                    .refs
                    .borrow()
                    .iter()
                    .for_each(|value| store.retain(*value));

                Instance::from_inner(store, instance)
            })
            .map_err(|e| error!("{}", e))
    }

    /// Validate that host functions defined via func_new match the component's expected import signatures
    fn validate_host_function_signatures(
        rb_self: &Obj<Self>,
        component: &Component,
    ) -> Result<(), Error> {
        // Get component type information
        let component_ty = component.get().component_type();
        let func_types = rb_self.func_types.borrow();
        let inner = rb_self.inner.borrow();
        let engine = inner.engine();

        // Helper to validate a single import
        fn validate_import(
            path: &str,
            name: &str,
            expected_func: &wasmtime::component::types::ComponentFunc,
            func_types: &HashMap<String, FuncTypeInfo>,
        ) -> Result<(), String> {
            let full_key = if path.is_empty() {
                name.to_string()
            } else {
                format!("{}/{}", path, name)
            };

            // Check if this function was defined in the linker
            if let Some(declared_info) = func_types.get(&full_key) {
                // Extract expected types from component
                let (expected_params, expected_results) = types::extract_func_types(expected_func)
                    .map_err(|e| format!("failed to extract function types: {}", e))?;

                // Validate parameter count
                if declared_info.param_types.len() != expected_params.len() {
                    return Err(format!(
                        "host function \"{}\" parameter count mismatch: declared has {} parameters, expected has {}",
                        full_key,
                        declared_info.param_types.len(),
                        expected_params.len()
                    ));
                }

                // Validate each parameter type
                for (i, (declared_ty, expected_ty)) in declared_info
                    .param_types
                    .iter()
                    .zip(expected_params.iter())
                    .enumerate()
                {
                    if let Err(e) = types::types_match(declared_ty, expected_ty) {
                        return Err(format!(
                            "host function \"{}\" parameter {} has incompatible type: {}",
                            full_key, i, e
                        ));
                    }
                }

                // Validate result count
                if declared_info.result_types.len() != expected_results.len() {
                    return Err(format!(
                        "host function \"{}\" result count mismatch: declared has {} results, expected has {}",
                        full_key,
                        declared_info.result_types.len(),
                        expected_results.len()
                    ));
                }

                // Validate each result type
                for (i, (declared_ty, expected_ty)) in declared_info
                    .result_types
                    .iter()
                    .zip(expected_results.iter())
                    .enumerate()
                {
                    if let Err(e) = types::types_match(declared_ty, expected_ty) {
                        return Err(format!(
                            "host function \"{}\" result {} has incompatible type: {}",
                            full_key, i, e
                        ));
                    }
                }
            }

            Ok(())
        }

        // Helper to recursively validate imports
        fn validate_imports_recursive(
            path: &str,
            item: &wasmtime::component::types::ComponentItem,
            func_types: &HashMap<String, FuncTypeInfo>,
            engine: &wasmtime::Engine,
        ) -> Result<(), String> {
            use wasmtime::component::types::ComponentItem;

            match item {
                ComponentItem::ComponentFunc(func) => {
                    // Extract just the function name from the full path
                    let name = if let Some(idx) = path.rfind('/') {
                        &path[idx + 1..]
                    } else {
                        path
                    };
                    let parent_path = if let Some(idx) = path.rfind('/') {
                        &path[..idx]
                    } else {
                        ""
                    };
                    validate_import(parent_path, name, func, func_types)?;
                }
                ComponentItem::ComponentInstance(instance) => {
                    // Recursively validate nested exports
                    for (export_name, export_item) in instance.exports(engine) {
                        let nested_path = if path.is_empty() {
                            export_name.to_string()
                        } else {
                            format!("{}/{}", path, export_name)
                        };
                        validate_imports_recursive(&nested_path, &export_item, func_types, engine)?;
                    }
                }
                _ => {
                    // Other types (Module, CoreFunc, Type, etc.) are not validated here
                }
            }
            Ok(())
        }

        // Validate all imports
        for (import_name, import_item) in component_ty.imports(engine) {
            validate_imports_recursive(import_name, &import_item, &func_types, engine)
                .map_err(|e| error!("{}", e))?;
        }

        Ok(())
    }

    pub(crate) fn add_wasi_p2(&self) -> Result<(), Error> {
        *self.has_wasi.borrow_mut() = true;
        let mut inner = self.inner.borrow_mut();
        wasmtime_wasi::p2::add_to_linker_sync(&mut inner).map_err(|e| error!("{e}"))
    }
}

/// @yard
/// @rename Wasmtime::Component::LinkerInstance
/// @see https://docs.rs/wasmtime/latest/wasmtime/component/struct.LinkerInstance.html Wasmtime's Rust doc
/// {LinkerInstance}s are builder-style, ephemeral objects that can only be used
/// within the block to which they get yielded. Calling methods outside of the
/// block will raise.
#[derive(TypedData)]
#[magnus(
    class = "Wasmtime::Component::LinkerInstance",
    size,
    mark,
    free_immediately,
    unsafe_generics
)]
pub struct LinkerInstance<'a> {
    inner: RefCell<MaybeInstanceImpl<'a>>,
    refs: RefCell<Vec<Value>>,
    path: String,         // namespace path, e.g., "" for root, "math" for nested
    parent_linker: Value, // Reference to parent Linker object for accessing func_types
}

unsafe impl Send for LinkerInstance<'_> {}

impl DataTypeFunctions for LinkerInstance<'_> {
    fn mark(&self, marker: &Marker) {
        marker.mark_slice(self.refs.borrow().as_slice());
        marker.mark(self.parent_linker);
    }
}

struct MaybeInstanceImpl<'a>(Option<LinkerInstanceImpl<'a, StoreData>>);
impl<'a> MaybeInstanceImpl<'a> {
    pub fn new(instance: LinkerInstanceImpl<'a, StoreData>) -> Self {
        Self(Some(instance))
    }

    pub fn get_mut(&mut self) -> Result<&mut LinkerInstanceImpl<'a, StoreData>, Error> {
        match &mut self.0 {
            Some(instance) => Ok(instance),
            None => err!("LinkerInstance went out of scope"),
        }
    }

    pub fn expire(&mut self) -> Option<LinkerInstanceImpl<'a, StoreData>> {
        self.0.take()
    }
}

impl<'a> LinkerInstance<'a> {
    fn from_inner(
        inner: LinkerInstanceImpl<'a, StoreData>,
        path: String,
        parent_linker: Value,
    ) -> Self {
        Self {
            inner: RefCell::new(MaybeInstanceImpl::new(inner)),
            refs: RefCell::new(Vec::new()),
            path,
            parent_linker,
        }
    }

    /// @yard
    /// @def module(name, mod)
    /// @param name [String]
    /// @param mod [Module]
    fn module(rb_self: Obj<Self>, name: RString, module: &Module) -> Result<Obj<Self>, Error> {
        let Ok(mut maybe_instance) = rb_self.inner.try_borrow_mut() else {
            return err!("LinkerInstance is not reentrant");
        };

        let inner = maybe_instance.get_mut()?;
        inner
            .module(unsafe { name.as_str()? }, module.get())
            .map_err(|e| error!("{}", e))?;

        Ok(rb_self)
    }

    /// @yard
    /// Defines a nested instance within the instance.
    /// @def instance(name)
    /// @param name [String]
    /// @yield [instance] The block allows configuring the {LinkerInstance};
    ///   outside of this scope the instance becomes unusable.
    /// @yieldparam instance [LinkerInstance]
    /// @return [LinkerInstance] +self+
    fn instance(ruby: &Ruby, rb_self: Obj<Self>, name: RString) -> Result<Obj<Self>, Error> {
        let Ok(mut maybe_instance) = rb_self.inner.try_borrow_mut() else {
            return err!("LinkerInstance is not reentrant");
        };

        let name_str = unsafe { name.as_str() }?;
        let inner = maybe_instance.get_mut()?;
        let nested_inner = inner.instance(name_str).map_err(|e| error!("{}", e))?;

        // Build nested path: if parent is "", use name; otherwise "parent/name"
        let nested_path = if rb_self.path.is_empty() {
            name_str.to_string()
        } else {
            format!("{}/{}", rb_self.path, name_str)
        };

        let nested_instance = ruby.obj_wrap(LinkerInstance::from_inner(
            nested_inner,
            nested_path,
            rb_self.parent_linker,
        ));
        let block_result: Result<Value, _> = ruby.yield_value(nested_instance);
        nested_instance.take_inner();

        match block_result {
            Ok(_) => Ok(rb_self),
            Err(e) => Err(e),
        }
    }

    /// @yard
    /// Define a host function in this linker instance.
    ///
    /// @def func_new(name, params, results, &block)
    /// @param name [String] The function name
    /// @param params [Array<Type>] The function parameter types
    /// @param results [Array<Type>] The function result types
    /// @yield [caller, *args] The block implementing the host function
    /// @yieldparam caller [Caller] The caller context (not yet fully implemented)
    /// @yieldparam args [Array<Object>] The function arguments, converted from component values
    /// @yieldreturn [Object, Array<Object>] The function result(s), will be validated and converted
    /// @return [LinkerInstance] +self+
    fn func_new(_ruby: &Ruby, rb_self: Obj<Self>, args: &[Value]) -> Result<Obj<Self>, Error> {
        let args = scan_args::<(RString, RArray, RArray), (), (), (), (), Proc>(args)?;
        let (name, params_array, results_array) = args.required;
        let callable = args.block;

        // Extract ComponentType from Type values
        let mut param_types = Vec::with_capacity(params_array.len());
        for param_value in unsafe { params_array.as_slice() } {
            param_types.push(types::extract_component_type(*param_value)?);
        }

        let mut result_types = Vec::with_capacity(results_array.len());
        for result_value in unsafe { results_array.as_slice() } {
            result_types.push(types::extract_component_type(*result_value)?);
        }

        let name_str = unsafe { name.as_str() }?;

        // Store type metadata for later validation
        let full_key = if rb_self.path.is_empty() {
            name_str.to_string()
        } else {
            format!("{}/{}", rb_self.path, name_str)
        };

        // Get parent Linker - we'll store the callable there to prevent GC
        // (rb_self is ephemeral and won't keep references alive after the block ends)
        let parent_linker: Obj<Linker> = Obj::try_convert(rb_self.parent_linker)?;

        parent_linker.refs.borrow_mut().push(callable.as_value());
        parent_linker.func_types.borrow_mut().insert(
            full_key,
            FuncTypeInfo {
                path: rb_self.path.clone(),
                param_types: param_types.clone(),
                result_types: result_types.clone(),
            },
        );

        // Create the closure that will be called from Wasm
        let func_closure =
            make_component_func_closure(param_types.clone(), result_types.clone(), callable.into());

        let Ok(mut maybe_instance) = rb_self.inner.try_borrow_mut() else {
            return err!("LinkerInstance is not reentrant");
        };

        let inner = maybe_instance.get_mut()?;
        inner
            .func_new(name_str, func_closure)
            .map_err(|e| error!("failed to define host function: {}", e))?;

        Ok(rb_self)
    }

    fn take_inner(&self) {
        let Ok(mut maybe_instance) = self.inner.try_borrow_mut() else {
            panic!("Linker instance is already borrowed, can't expire.")
        };

        maybe_instance.expire();
    }
}

/// Create a closure that wraps a Ruby Proc for use as a component host function
fn make_component_func_closure(
    param_types: Vec<ComponentType>,
    result_types: Vec<ComponentType>,
    callable: Opaque<Proc>,
) -> impl Fn(
    wasmtime::StoreContextMut<'_, StoreData>,
    wasmtime::component::types::ComponentFunc,
    &[Val],
    &mut [Val],
) -> wasmtime::Result<()>
       + Send
       + Sync
       + 'static {
    move |mut store_context: wasmtime::StoreContextMut<'_, StoreData>,
          _func: wasmtime::component::types::ComponentFunc,
          params: &[Val],
          results: &mut [Val]| {
        let ruby = Ruby::get().unwrap();

        // Convert Wasm params to Ruby values
        let rparams = ruby.ary_new_capa(params.len());
        for (i, (param, _param_ty)) in params.iter().zip(param_types.iter()).enumerate() {
            let rb_value =
                convert::component_val_to_rb(&ruby, param.clone(), None).map_err(|e| {
                    wasmtime::Error::msg(format!("failed to convert parameter at index {i}: {e}"))
                })?;
            rparams.push(rb_value).map_err(|e| {
                wasmtime::Error::msg(format!("failed to push parameter at index {i}: {e}"))
            })?;
        }

        // Call the Ruby Proc
        let callable = ruby.get_inner(callable);
        let proc_result = callable.call::<_, Value>(rparams).map_err(|e| {
            // Store the Ruby error on StoreData so it can be properly raised later
            store_context.data_mut().set_error(e);
            // Return a generic error that will be replaced with the Ruby error
            wasmtime::Error::msg("")
        })?;

        // Handle result conversion based on arity
        match result_types.len() {
            0 => {
                // No return value expected
                Ok(())
            }
            1 => {
                // Single return value - accept either the value directly or in an array
                let result_value = if let Ok(result_array) = RArray::to_ary(proc_result) {
                    if result_array.len() != 1 {
                        return Err(wasmtime::Error::msg(format!(
                            "expected 1 result, got {}",
                            result_array.len()
                        )));
                    }
                    unsafe { result_array.as_slice()[0] }
                } else {
                    proc_result
                };

                let converted = convert::validate_and_convert(result_value, None, &result_types[0])
                    .map_err(|e| {
                        // Store type errors on StoreData as well
                        store_context.data_mut().set_error(e);
                        wasmtime::Error::msg("")
                    })?;
                results[0] = converted;
                Ok(())
            }
            n => {
                // Multiple return values - expect an array
                let result_array = RArray::to_ary(proc_result)
                    .map_err(|_| wasmtime::Error::msg("expected array of results"))?;

                if result_array.len() != n {
                    return Err(wasmtime::Error::msg(format!(
                        "expected {} results, got {}",
                        n,
                        result_array.len()
                    )));
                }

                for (i, (result_value, result_ty)) in unsafe { result_array.as_slice() }
                    .iter()
                    .zip(result_types.iter())
                    .enumerate()
                {
                    let converted = convert::validate_and_convert(*result_value, None, result_ty)
                        .map_err(|e| {
                        // Store type errors on StoreData as well
                        store_context.data_mut().set_error(e);
                        wasmtime::Error::msg("")
                    })?;
                    results[i] = converted;
                }
                Ok(())
            }
        }
    }
}

pub fn init(ruby: &Ruby, namespace: &RModule) -> Result<(), Error> {
    let linker = namespace.define_class("Linker", ruby.class_object())?;
    linker.define_singleton_method("new", function!(Linker::new, 1))?;
    linker.define_method("root", method!(Linker::root, 0))?;
    linker.define_method("instance", method!(Linker::instance, 1))?;
    linker.define_method("instantiate", method!(Linker::instantiate, 2))?;

    let linker_instance = namespace.define_class("LinkerInstance", ruby.class_object())?;
    linker_instance.define_method("module", method!(LinkerInstance::module, 2))?;
    linker_instance.define_method("instance", method!(LinkerInstance::instance, 1))?;
    linker_instance.define_method("func_new", method!(LinkerInstance::func_new, -1))?;

    Ok(())
}
