use super::{
    convert::WrapWasmtimeType,
    engine::Engine,
    func::{self, Func},
    func_type::FuncType,
    instance::Instance,
    module::Module,
    root,
    store::{Store, StoreData},
};
use crate::{error, ruby_api::convert::ToExtern};
use magnus::{
    block::Proc,
    exception::arg_error,
    function, gc, method,
    scan_args::{get_kwargs, scan_args},
    DataTypeFunctions, Error, Module as _, Object, RHash, RString, TypedData, Value,
};
use std::cell::RefCell;
use wasmtime::Linker as LinkerImpl;

#[derive(TypedData)]
#[magnus(class = "Wasmtime::Linker", size, mark, free_immediatly)]
pub struct Linker {
    inner: RefCell<LinkerImpl<StoreData>>,
    refs: RefCell<Vec<Value>>,
}

unsafe impl Send for Linker {}

impl DataTypeFunctions for Linker {
    fn mark(&self) {
        self.refs.borrow().iter().for_each(gc::mark);
    }
}

impl Linker {
    pub fn new(args: &[Value]) -> Result<Self, Error> {
        let args = scan_args::<(&Engine,), (), (), (), RHash, ()>(args)?;
        let (engine,) = args.required;

        Ok(Self {
            inner: RefCell::new(LinkerImpl::new(engine.get())),
            refs: Default::default(),
        })
    }

    pub fn set_allow_shadowing(&self, val: bool) {
        self.inner.borrow_mut().allow_shadowing(val);
    }

    pub fn set_allow_unknown_exports(&self, val: bool) {
        self.inner.borrow_mut().allow_unknown_exports(val);
    }

    pub fn define_unknown_imports_as_traps(&self, module: &Module) -> Result<(), Error> {
        self.inner
            .borrow_mut()
            .define_unknown_imports_as_traps(module.get())
            .map_err(|e| error!("{}", e))
    }

    pub fn define(&self, module: RString, name: RString, item: Value) -> Result<(), Error> {
        let item = item.to_extern()?;

        self.inner
            .borrow_mut()
            .define(unsafe { module.as_str()? }, unsafe { name.as_str()? }, item)
            .map(|_| ())
            .map_err(|e| error!("{}", e))
    }

    pub fn func_new(&self, args: &[Value]) -> Result<(), Error> {
        let args = scan_args::<
            (RString, RString, &FuncType),
            (Option<Proc>,),
            (),
            (),
            RHash,
            Option<Proc>,
        >(args)?;
        let (module, name, ty) = args.required;
        let (proc,) = args.optional;
        let block = args.block;
        let kwargs = get_kwargs::<_, (), (Option<bool>,), ()>(args.keywords, &[], &["caller"])?;
        let (send_caller,) = kwargs.optional;
        let send_caller = send_caller.unwrap_or(false);

        if proc.and(block).is_some() {
            return Err(Error::new(
                arg_error(),
                "provide block or proc argument, not both",
            ));
        }
        let proc = proc
            .or(block)
            .ok_or_else(|| Error::new(arg_error(), "provide block or proc argument"))?;

        let func_callable = func::make_func_callable(ty.get(), proc, send_caller);

        self.refs.borrow_mut().push(proc.into());

        self.inner
            .borrow_mut()
            .func_new(
                unsafe { module.as_str() }?,
                unsafe { name.as_str() }?,
                ty.get().clone(),
                func_callable,
            )
            .map_err(|e| error!("{}", e))
            .map(|_| ())
    }

    pub fn get(&self, s: Value, module: RString, name: RString) -> Result<Option<Value>, Error> {
        let store: &Store = s.try_convert()?;
        let ext =
            self.inner
                .borrow()
                .get(store.context_mut(), unsafe { module.as_str() }?, unsafe {
                    name.as_str()?
                });

        match ext {
            None => Ok(None),
            Some(ext) => ext.wrap_wasmtime_type(s).map(Some),
        }
    }

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

    pub fn module(&self, store: &Store, name: RString, module: &Module) -> Result<(), Error> {
        self.inner
            .borrow_mut()
            .module(store.context_mut(), unsafe { name.as_str()? }, module.get())
            .map(|_| ())
            .map_err(|e| error!("{}", e))
    }

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

    pub fn alias_module(&self, module: RString, as_module: RString) -> Result<(), Error> {
        self.inner
            .borrow_mut()
            .alias_module(unsafe { module.as_str() }?, unsafe { as_module.as_str() }?)
            .map_err(|e| error!("{}", e))
            .map(|_| ())
    }

    pub fn instantiate(&self, s: Value, module: &Module) -> Result<Instance, Error> {
        let store = s.try_convert::<&Store>()?;
        self.inner
            .borrow_mut()
            .instantiate(store.context_mut(), module.get())
            .map_err(|e| {
                store
                    .context_mut()
                    .data_mut()
                    .take_last_error()
                    .unwrap_or_else(|| error!("{}", e))
            })
            .map(|instance| {
                self.refs.borrow().iter().for_each(|val| store.retain(*val));
                Instance::from_inner(s, instance)
            })
    }

    pub fn get_default(&self, s: Value, module: RString) -> Result<Func, Error> {
        let store: &Store = s.try_convert()?;
        self.inner
            .borrow()
            .get_default(store.context_mut(), unsafe { module.as_str() }?)
            .map(|func| Func::from_inner(s, func))
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
