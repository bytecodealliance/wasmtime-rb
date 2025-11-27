use super::{
    convert::WrapWasmtimeType,
    convert::{ToExtern, ToValTypeVec},
    engine::Engine,
    externals::Extern,
    func::{self, Func},
    instance::Instance,
    module::Module,
    root,
    store::{Store, StoreContextValue, StoreData},
};
use crate::{err, error, ruby_api::errors};
use magnus::{
    block::Proc, class, function, gc::Marker, method, prelude::*, scan_args, scan_args::scan_args,
    typed_data::Obj, DataTypeFunctions, Error, Object, RArray, RHash, RString, Ruby, TypedData,
    Value,
};
use std::cell::RefCell;
use wasmtime::Linker as LinkerImpl;

/// @yard
/// @see https://docs.rs/wasmtime/latest/wasmtime/struct.Linker.html Wasmtime's Rust doc
#[derive(TypedData)]
#[magnus(class = "Wasmtime::Linker", size, mark, free_immediately)]
pub struct Linker {
    inner: RefCell<LinkerImpl<StoreData>>,
    refs: RefCell<Vec<Value>>,
    has_wasi: RefCell<bool>,
}

unsafe impl Send for Linker {}

impl DataTypeFunctions for Linker {
    fn mark(&self, marker: &Marker) {
        marker.mark_slice(self.refs.borrow().as_slice());
    }
}

impl Linker {
    /// @yard
    /// @def new(engine)
    /// @param engine [Engine]
    /// @return [Linker]
    pub fn new(engine: &Engine) -> Result<Self, Error> {
        let inner: LinkerImpl<StoreData> = LinkerImpl::new(engine.get());
        Ok(Self {
            inner: RefCell::new(inner),
            refs: Default::default(),
            has_wasi: RefCell::new(false),
        })
    }

    /// @yard
    /// Allow shadowing.
    /// @def allow_shadowing=(val)
    /// @param val [Boolean]
    pub fn set_allow_shadowing(&self, val: bool) {
        self.inner.borrow_mut().allow_shadowing(val);
    }

    /// @yard
    /// Allow unknown exports.
    /// @def allow_unknown_exports=(val)
    /// @param val [Boolean]
    pub fn set_allow_unknown_exports(&self, val: bool) {
        self.inner.borrow_mut().allow_unknown_exports(val);
    }

    /// @yard
    /// Define unknown (unresolved) imports as functions which trap.
    /// @def define_unknown_imports_as_traps(mod)
    /// @param mod [Module]
    /// @return [void]
    pub fn define_unknown_imports_as_traps(&self, module: &Module) -> Result<(), Error> {
        self.inner
            .borrow_mut()
            .define_unknown_imports_as_traps(module.get())
            .map_err(|e| error!("{}", e))
    }

    /// @yard
    /// Define an item in this linker.
    /// @def define(store, mod, name, item)
    /// @param store [Store]
    /// @param mod [String] Module name
    /// @param name [String] Import name
    /// @param item [Func, Memory] The item to define.
    /// @return [void]
    pub fn define(
        ruby: &Ruby,
        rb_self: &Self,
        store: &Store,
        module: RString,
        name: RString,
        item: Value,
    ) -> Result<(), Error> {
        let item = item.to_extern(ruby)?;

        rb_self
            .inner
            .borrow_mut()
            .define(
                store.context(),
                unsafe { module.as_str()? },
                unsafe { name.as_str()? },
                item,
            )
            .map(|_| ())
            .map_err(|e| error!("{}", e))
    }

    /// @yard
    /// Define a function in this linker.
    ///
    /// @see Wasmtime::Func.new
    ///
    /// @def func_new(mod, name, params, results, &block)
    /// @param mod [String] Module name
    /// @param name [String] Import name
    /// @param params [Array<Symbol>] The function's parameters.
    /// @param results [Array<Symbol>] The function's results.
    /// @param block [Block] See {Func.new} for block argument details.
    /// @return [void]
    /// @see Func.new
    pub fn func_new(&self, args: &[Value]) -> Result<(), Error> {
        let args = scan_args::<(RString, RString, RArray, RArray), (), (), (), RHash, Proc>(args)?;
        let (module, name, params, results) = args.required;
        let callable = args.block;
        let mut inner_mut = self.inner.borrow_mut();
        let engine = inner_mut.engine();
        let ty = wasmtime::FuncType::new(
            engine,
            params.to_val_type_vec()?,
            results.to_val_type_vec()?,
        );
        let func_closure = func::make_func_closure(&ty, callable.into());

        self.refs.borrow_mut().push(callable.as_value());

        inner_mut
            .func_new(
                unsafe { module.as_str() }?,
                unsafe { name.as_str() }?,
                ty,
                func_closure,
            )
            .map_err(|e| error!("{}", e))
            .map(|_| ())
    }

    /// @yard
    /// Looks up a previously defined item in this linker.
    ///
    /// @def get(store, mod, name)
    /// @param store [Store]
    /// @param mod [String] Module name
    /// @param name [String] Import name
    /// @return [Extern, nil] The item if it exists, nil otherwise.
    pub fn get(
        &self,
        store: Obj<Store>,
        module: RString,
        name: RString,
    ) -> Result<Option<Extern<'_>>, Error> {
        let ext =
            self.inner
                .borrow()
                .get(store.context_mut(), unsafe { module.as_str() }?, unsafe {
                    name.as_str()?
                });

        match ext {
            None => Ok(None),
            Some(ext) => ext.wrap_wasmtime_type(store.into()).map(Some),
        }
    }

    /// @yard
    /// Defines an entire {Instance} in this linker.
    ///
    /// @def instance(store, mod, instance)
    /// @param store [Store]
    /// @param mod [String] Module name
    /// @param instance [Instance]
    /// @return [void]
    pub fn instance(
        &self,
        store: &Store,
        module: RString,
        instance: &Instance,
    ) -> Result<(), Error> {
        self.inner
            .borrow_mut()
            .instance(
                store.context_mut(),
                unsafe { module.as_str() }?,
                instance.get(),
            )
            .map_err(|e| error!("{}", e))
            .map(|_| ())
    }

    /// @yard
    /// Defines automatic instantiation of a {Module} in this linker.
    ///
    /// @def module(store, name, mod)
    /// @param store [Store]
    /// @param name [String] Module name
    /// @param mod [Module]
    /// @return [void]
    pub fn module(&self, store: &Store, name: RString, module: &Module) -> Result<(), Error> {
        self.inner
            .borrow_mut()
            .module(store.context_mut(), unsafe { name.as_str()? }, module.get())
            .map(|_| ())
            .map_err(|e| error!("{}", e))
    }

    /// @yard
    /// Aliases one item’s name as another.
    ///
    /// @def alias(mod, name, as_mod, as_name)
    /// @param mod [String] The source module name.
    /// @param name [String] The source item name.
    /// @param as_mod [String] The destination module name.
    /// @param as_name [String] The destination item name.
    /// @return [void]
    pub fn alias(
        &self,
        module: RString,
        name: RString,
        as_module: RString,
        as_name: RString,
    ) -> Result<(), Error> {
        self.inner
            .borrow_mut()
            .alias(
                unsafe { module.as_str() }?,
                unsafe { name.as_str() }?,
                unsafe { as_module.as_str() }?,
                unsafe { as_name.as_str() }?,
            )
            .map_err(|e| error!("{}", e))
            .map(|_| ())
    }

    /// @yard
    /// Aliases one module’s name as another.
    ///
    /// @def alias(mod, as_mod)
    /// @param mod [String] Source module name
    /// @param as_mod [String] Destination module name
    /// @return [void]
    pub fn alias_module(&self, module: RString, as_module: RString) -> Result<(), Error> {
        self.inner
            .borrow_mut()
            .alias_module(unsafe { module.as_str() }?, unsafe { as_module.as_str() }?)
            .map_err(|e| error!("{}", e))
            .map(|_| ())
    }

    /// @yard
    /// Instantiates a {Module} in a {Store} using the defined imports in the linker.
    /// @def instantiate(store, mod)
    /// @param store [Store]
    /// @param mod [Module]
    /// @return [Instance]
    pub fn instantiate(
        ruby: &Ruby,
        rb_self: Obj<Self>,
        store: Obj<Store>,
        module: &Module,
    ) -> Result<Instance, Error> {
        if *rb_self.has_wasi.borrow() && !store.context().data().has_wasi_p1_ctx() {
            return err!("{}", errors::missing_wasi_p1_ctx_error());
        }

        rb_self
            .inner
            .borrow_mut()
            .instantiate(store.context_mut(), module.get())
            .map_err(|e| StoreContextValue::from(store).handle_wasm_error(ruby, e))
            .map(|instance| {
                rb_self
                    .refs
                    .borrow()
                    .iter()
                    .for_each(|val| store.retain(*val));
                Instance::from_inner(store, instance)
            })
    }

    /// @yard
    /// Returns the “default export” of a module.
    /// @def get_default(store, mod)
    /// @param store [Store]
    /// @param mod [String] Module name
    /// @return [Func]
    pub fn get_default(&self, store: Obj<Store>, module: RString) -> Result<Func<'_>, Error> {
        self.inner
            .borrow()
            .get_default(store.context_mut(), unsafe { module.as_str() }?)
            .map(|func| Func::from_inner(store.into(), func))
            .map_err(|e| error!("{}", e))
    }

    /// @yard
    /// Replaces the `poll_oneoff` and `sched_yield` function implementations
    /// with deterministic ones.
    /// @return [void]
    pub fn use_deterministic_scheduling_functions(&self) -> Result<(), Error> {
        let mut inner = self.inner.borrow_mut();
        deterministic_wasi_ctx::replace_scheduling_functions(&mut inner).map_err(|e| error!("{e}"))
    }

    pub(crate) fn add_wasi_p1(&self) -> Result<(), Error> {
        *self.has_wasi.borrow_mut() = true;
        let mut inner = self.inner.borrow_mut();
        wasmtime_wasi::p1::add_to_linker_sync(&mut inner, |s| s.wasi_p1_ctx_mut())
            .map_err(|e| error!("{e}"))
    }
}

pub fn init(ruby: &Ruby) -> Result<(), Error> {
    let class = root().define_class("Linker", ruby.class_object())?;
    class.define_singleton_method("new", function!(Linker::new, 1))?;
    class.define_method("allow_shadowing=", method!(Linker::set_allow_shadowing, 1))?;
    class.define_method(
        "allow_unknown_exports=",
        method!(Linker::set_allow_unknown_exports, 1),
    )?;
    class.define_method(
        "define_unknown_imports_as_traps",
        method!(Linker::define_unknown_imports_as_traps, 1),
    )?;
    class.define_method("define", method!(Linker::define, 4))?;
    class.define_method("func_new", method!(Linker::func_new, -1))?;
    class.define_method("get", method!(Linker::get, 3))?;
    class.define_method("instance", method!(Linker::instance, 3))?;
    class.define_method("module", method!(Linker::module, 3))?;
    class.define_method("alias", method!(Linker::alias, 4))?;
    class.define_method("alias_module", method!(Linker::alias_module, 2))?;
    class.define_method("instantiate", method!(Linker::instantiate, 2))?;
    class.define_method("get_default", method!(Linker::get_default, 2))?;
    class.define_method(
        "use_deterministic_scheduling_functions",
        method!(Linker::use_deterministic_scheduling_functions, 0),
    )?;

    Ok(())
}
