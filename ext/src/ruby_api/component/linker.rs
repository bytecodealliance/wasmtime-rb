use super::{Component, Instance};
use crate::{
    define_rb_intern, err,
    ruby_api::{
        errors,
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
    class, function, gc::Marker, method, r_string::RString, scan_args, typed_data::Obj,
    DataTypeFunctions, Error, Module as _, Object, RModule, Ruby, TryConvert, TypedData, Value,
};
use wasmtime::component::{Linker as LinkerImpl, LinkerInstance as LinkerInstanceImpl};
use wasmtime_wasi::{
    p2::{IoView, WasiCtx, WasiView},
    ResourceTable,
};

define_rb_intern!(
    WASI => "wasi",
);

/// @yard
/// @rename Wasmtime::Component::Linker
/// @see https://docs.rs/wasmtime/latest/wasmtime/component/struct.Linker.html Wasmtime's Rust doc
#[derive(TypedData)]
#[magnus(class = "Wasmtime::Component::Linker", size, mark, free_immediately)]
pub struct Linker {
    inner: RefCell<LinkerImpl<StoreData>>,
    refs: RefCell<Vec<Value>>,
    has_wasi: bool,
}
unsafe impl Send for Linker {}

impl DataTypeFunctions for Linker {
    fn mark(&self, marker: &magnus::gc::Marker) {
        marker.mark_slice(self.refs.borrow().as_slice());
    }
}

impl Linker {
    /// @yard
    /// @def new(engine, wasi: false)
    /// @param engine [Engine]
    /// @param wasi [Boolean] Whether WASI should be defined in this Linker. Defaults to false.
    /// @return [Linker]
    pub fn new(args: &[Value]) -> Result<Self, Error> {
        let args = scan_args::scan_args::<(&Engine,), (), (), (), _, ()>(args)?;
        let kw = scan_args::get_kwargs::<_, (), (Option<bool>,), ()>(args.keywords, &[], &[*WASI])?;
        let (engine,) = args.required;
        let wasi = kw.optional.0.unwrap_or(false);

        let mut linker: LinkerImpl<StoreData> = LinkerImpl::new(engine.get());
        if wasi {
            wasmtime_wasi::p2::add_to_linker_sync(&mut linker).map_err(|e| error!("{}", e))?
        }

        Ok(Linker {
            inner: RefCell::new(linker),
            refs: RefCell::new(Vec::new()),
            has_wasi: wasi,
        })
    }

    pub(crate) fn inner_mut(&self) -> RefMut<LinkerImpl<StoreData>> {
        self.inner.borrow_mut()
    }

    pub(crate) fn has_wasi(&self) -> bool {
        self.has_wasi
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
        let instance = Obj::wrap(LinkerInstance::from_inner(inner.root()));
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
        let instance = inner
            .instance(unsafe { name.as_str() }?)
            .map_err(|e| error!("{}", e))?;

        let instance = Obj::wrap(LinkerInstance::from_inner(instance));

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
        if rb_self.has_wasi && !store.context().data().has_wasi_ctx() {
            return err!("{}", errors::missing_wasi_ctx_error());
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
}

unsafe impl Send for LinkerInstance<'_> {}

impl DataTypeFunctions for LinkerInstance<'_> {
    fn mark(&self, marker: &Marker) {
        marker.mark_slice(self.refs.borrow().as_slice());
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
    fn from_inner(inner: LinkerInstanceImpl<'a, StoreData>) -> Self {
        Self {
            inner: RefCell::new(MaybeInstanceImpl::new(inner)),
            refs: RefCell::new(Vec::new()),
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

        let inner = maybe_instance.get_mut()?;
        let nested_inner = inner
            .instance(unsafe { name.as_str()? })
            .map_err(|e| error!("{}", e))?;

        let nested_instance = Obj::wrap(LinkerInstance::from_inner(nested_inner));
        let block_result: Result<Value, _> = ruby.yield_value(nested_instance);
        nested_instance.take_inner();

        match block_result {
            Ok(_) => Ok(rb_self),
            Err(e) => Err(e),
        }
    }

    fn take_inner(&self) {
        let Ok(mut maybe_instance) = self.inner.try_borrow_mut() else {
            panic!("Linker instance is already borrowed, can't expire.")
        };

        maybe_instance.expire();
    }
}

pub fn init(_ruby: &Ruby, namespace: &RModule) -> Result<(), Error> {
    let linker = namespace.define_class("Linker", class::object())?;
    linker.define_singleton_method("new", function!(Linker::new, -1))?;
    linker.define_method("root", method!(Linker::root, 0))?;
    linker.define_method("instance", method!(Linker::instance, 1))?;
    linker.define_method("instantiate", method!(Linker::instantiate, 2))?;

    let linker_instance = namespace.define_class("LinkerInstance", class::object())?;
    linker_instance.define_method("module", method!(LinkerInstance::module, 2))?;
    linker_instance.define_method("instance", method!(LinkerInstance::instance, 1))?;

    Ok(())
}
