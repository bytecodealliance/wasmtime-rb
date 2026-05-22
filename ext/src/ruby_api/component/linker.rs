use super::convert;
use super::{Component, Instance};
use crate::{
    err,
    ruby_api::{
        errors::{self, ExceptionMessage},
        store::{StoreContextValue, StoreData},
        Engine, Module, Store,
    },
};
use std::{
    borrow::BorrowMut,
    cell::{RefCell, RefMut},
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

/// @yard
/// @rename Wasmtime::Component::Linker
/// @see https://docs.rs/wasmtime/latest/wasmtime/component/struct.Linker.html Wasmtime's Rust doc
#[derive(TypedData)]
#[magnus(class = "Wasmtime::Component::Linker", size, mark, free_immediately)]
pub struct Linker {
    inner: RefCell<LinkerImpl<StoreData>>,
    refs: RefCell<Vec<Value>>,
    has_wasi: RefCell<bool>,
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
    /// Host functions return plain Ruby values which are automatically validated
    /// and converted based on the function's type signature from the component.
    ///
    /// @example Simple scalar return
    ///   root.func_new("add") do |a, b|
    ///     a + b  # Returns u32
    ///   end
    ///
    /// @example Returning a list
    ///   root.func_new("get-numbers") do
    ///     [1, 2, 3]  # Returns list<s32>
    ///   end
    ///
    /// @example Returning a tuple
    ///   root.func_new("make-tuple") do |n, s, b|
    ///     [n, s, b]  # Returns tuple<u32, string, bool>
    ///   end
    ///
    /// @example Complex types (records, results, etc.)
    ///   root.func_new("make-point") do |x, y|
    ///     {"x" => x, "y" => y}  # Returns record with x and y fields
    ///   end
    ///
    /// @def func_new(name, &block)
    /// @param name [String] The function name
    /// @yield [*args] The block implementing the host function
    /// @yieldparam args [Array<Object>] The function arguments, converted from component values
    /// @yieldreturn [Object] Result value matching the function's return type. Use arrays for lists and tuples, hashes for records.
    /// @return [LinkerInstance] +self+
    fn func_new(_ruby: &Ruby, rb_self: Obj<Self>, args: &[Value]) -> Result<Obj<Self>, Error> {
        let args = scan_args::<(RString,), (), (), (), (), Proc>(args)?;
        let (name,) = args.required;
        let callable = args.block;

        let name_str = unsafe { name.as_str() }?;

        // Get parent Linker - we'll store the callable there to prevent GC
        // (rb_self is ephemeral and won't keep references alive after the block ends)
        let parent_linker: Obj<Linker> = Obj::try_convert(rb_self.parent_linker)?;
        parent_linker.refs.borrow_mut().push(callable.as_value());

        // Create the closure that will be called from Wasm
        let func_closure = make_component_func_closure(callable.into());

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
/// The closure uses the function's type signature for automatic validation and conversion
fn make_component_func_closure(
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
          func: wasmtime::component::types::ComponentFunc,
          params: &[Val],
          results: &mut [Val]| {
        let ruby = Ruby::get().unwrap();

        // Convert Wasm params to Ruby values
        let rparams = ruby.ary_new_capa(params.len());
        for (i, param) in params.iter().enumerate() {
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

        // Get expected result types from function signature
        let results_types: Vec<_> = func.results().collect();
        let num_results = results_types.len();

        // Handle result conversion based on arity
        // Note: WIT only supports 0 or 1 return values (use tuples for multiple values)
        match num_results {
            0 => {
                // No return value expected
                Ok(())
            }
            1 => {
                // Single return value - convert directly
                // Don't unwrap arrays - the value might be a list or tuple type
                let expected_ty = &results_types[0];
                let converted = convert::rb_to_component_val(proc_result, None, expected_ty)
                    .map_err(|e| {
                        store_context.data_mut().set_error(e);
                        wasmtime::Error::msg("")
                    })?;
                results[0] = converted;
                Ok(())
            }
            _ => {
                // WIT doesn't support multiple return values - this should never happen
                store_context.data_mut().set_error(Error::new(
                    ruby.exception_runtime_error(),
                    format!("unexpected number of results: {}", num_results),
                ));
                Err(wasmtime::Error::msg(""))
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
