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
use crate::{define_rb_intern, err, error, helpers::WrappedStruct};
use magnus::{
    block::Proc, function, gc, method, scan_args, scan_args::scan_args, DataTypeFunctions, Error,
    Module as _, Object, RArray, RHash, RString, TypedData, Value,
};
use std::cell::RefCell;
use wasmtime::Linker as LinkerImpl;

define_rb_intern!(
    WASI=> "wasi",
);

/// @yard
/// @see https://docs.rs/wasmtime/latest/wasmtime/struct.Linker.html Wasmtime's Rust doc
#[derive(TypedData)]
#[magnus(class = "Wasmtime::Linker", size, mark, free_immediatly)]
pub struct Linker {
    inner: RefCell<LinkerImpl<StoreData>>,
    refs: RefCell<Vec<Value>>,
    has_wasi: bool,
}

unsafe impl Send for Linker {}

impl DataTypeFunctions for Linker {
    fn mark(&self) {
        gc::mark_slice(self.refs.borrow().as_slice());
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

        let mut inner: LinkerImpl<StoreData> = LinkerImpl::new(engine.get());
        if wasi {
            wasmtime_wasi::add_to_linker(&mut inner, |s| s.wasi_ctx_mut())
                .map_err(|e| error!("{}", e))?
        }
        Ok(Self {
            inner: RefCell::new(inner),
            refs: Default::default(),
            has_wasi: wasi,
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
    /// @def define(mod, name, item)
    /// @param mod [String] Module name
    /// @param name [String] Import name
    /// @param item [Func, Memory] The item to define.
    /// @return [void]
    pub fn define(&self, module: RString, name: RString, item: Value) -> Result<(), Error> {
        let item = item.to_extern()?;

        self.inner
            .borrow_mut()
            .define(unsafe { module.as_str()? }, unsafe { name.as_str()? }, item)
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
        let ty = wasmtime::FuncType::new(params.to_val_type_vec()?, results.to_val_type_vec()?);
        let func_closure = func::make_func_closure(&ty, callable);

        self.refs.borrow_mut().push(callable.into());

        self.inner
            .borrow_mut()
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
        s: WrappedStruct<Store>,
        module: RString,
        name: RString,
    ) -> Result<Option<Extern>, Error> {
        let store: &Store = s.try_convert()?;
        let ext =
            self.inner
                .borrow()
                .get(store.context_mut(), unsafe { module.as_str() }?, unsafe {
                    name.as_str()?
                });

        match ext {
            None => Ok(None),
            Some(ext) => ext.wrap_wasmtime_type(s.into()).map(Some),
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
    pub fn instantiate(&self, s: WrappedStruct<Store>, module: &Module) -> Result<Instance, Error> {
        let wrapped_store: WrappedStruct<Store> = s.try_convert()?;
        let store = wrapped_store.get()?;

        if self.has_wasi && !store.context().data().has_wasi_ctx() {
            return err!(
                "Store is missing WASI configuration.\n\n\
                When using `wasi: true`, the Store given to\n\
                `Linker#instantiate` must have a WASI configuration.\n\
                To fix this, provide the `wasi_ctx` when creating the Store:\n\
                    Wasmtime::Store.new(engine, wasi_ctx: WasiCtxBuilder.new)"
            );
        }

        self.inner
            .borrow_mut()
            .instantiate(store.context_mut(), module.get())
            .map_err(|e| StoreContextValue::from(wrapped_store).handle_wasm_error(e))
            .map(|instance| {
                self.refs.borrow().iter().for_each(|val| store.retain(*val));
                Instance::from_inner(s, instance)
            })
    }

    /// @yard
    /// Returns the “default export” of a module.
    /// @def get_default(store, mod)
    /// @param store [Store]
    /// @param mod [String] Module name
    /// @return [Func]
    pub fn get_default(&self, s: WrappedStruct<Store>, module: RString) -> Result<Func, Error> {
        let store = s.get()?;

        self.inner
            .borrow()
            .get_default(store.context_mut(), unsafe { module.as_str() }?)
            .map(|func| Func::from_inner(s.into(), func))
            .map_err(|e| error!("{}", e))
    }
}

pub fn init() -> Result<(), Error> {
    let class = root().define_class("Linker", Default::default())?;
    class.define_singleton_method("new", function!(Linker::new, -1))?;
    class.define_method("allow_shadowing=", method!(Linker::set_allow_shadowing, 1))?;
    class.define_method(
        "allow_unknown_exports=",
        method!(Linker::set_allow_unknown_exports, 1),
    )?;
    class.define_method(
        "define_unknown_imports_as_traps",
        method!(Linker::define_unknown_imports_as_traps, 1),
    )?;
    class.define_method("define", method!(Linker::define, 3))?;
    class.define_method("func_new", method!(Linker::func_new, -1))?;
    class.define_method("get", method!(Linker::get, 3))?;
    class.define_method("instance", method!(Linker::instance, 3))?;
    class.define_method("module", method!(Linker::module, 3))?;
    class.define_method("alias", method!(Linker::alias, 4))?;
    class.define_method("alias_module", method!(Linker::alias_module, 2))?;
    class.define_method("instantiate", method!(Linker::instantiate, 2))?;
    class.define_method("get_default", method!(Linker::get_default, 2))?;

    Ok(())
}
