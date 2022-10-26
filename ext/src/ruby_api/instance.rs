use super::{
    convert::{ToExtern, WrapWasmtimeType},
    func::Func,
    module::Module,
    root,
    store::{Store, StoreContextValue, StoreData},
};
use crate::{err, error};
use magnus::{
    function, gc, method, scan_args, DataTypeFunctions, Error, Module as _, Object, RArray, RHash,
    RString, TypedData, Value,
};
use wasmtime::{Extern, Instance as InstanceImpl, StoreContextMut};

#[derive(Clone, Debug, TypedData)]
#[magnus(class = "Wasmtime::Instance", mark, free_immediatly)]
pub struct Instance {
    inner: InstanceImpl,
    store: Value,
}

unsafe impl Send for Instance {}

impl DataTypeFunctions for Instance {
    fn mark(&self) {
        gc::mark(&self.store);
    }
}

impl Instance {
    pub fn new(args: &[Value]) -> Result<Self, Error> {
        let args =
            scan_args::scan_args::<(Value, &Module), (Option<Value>,), (), (), (), ()>(args)?;
        let (s, module) = args.required;
        let store: &Store = s.try_convert()?;
        let context = store.context_mut();
        let imports = args
            .optional
            .0
            .and_then(|v| if v.is_nil() { None } else { Some(v) });

        let imports: Vec<Extern> = match imports {
            Some(arr) => {
                let arr: RArray = arr.try_convert()?;
                let mut imports = Vec::with_capacity(arr.len());
                for import in arr.each() {
                    let import = import?;
                    store.retain(import);
                    imports.push(import.to_extern()?);
                }
                imports
            }
            None => vec![],
        };

        let module = module.get();
        let inner = InstanceImpl::new(context, module, &imports).map_err(|e| {
            store
                .context_mut()
                .data_mut()
                .take_last_error()
                .unwrap_or_else(|| error!("{}", e))
        })?;

        Ok(Self { inner, store: s })
    }

    pub fn get(&self) -> InstanceImpl {
        self.inner
    }

    pub fn from_inner(store: Value, inner: InstanceImpl) -> Self {
        Self { inner, store }
    }

    pub fn exports(&self) -> Result<RHash, Error> {
        let store = self.store.try_convert::<&Store>()?;
        let mut ctx = store.context_mut();
        let hash = RHash::new();

        for export in self.inner.exports(&mut ctx) {
            hash.aset(
                RString::from(export.name()),
                export.into_extern().wrap_wasmtime_type(self.store)?,
            )?;
        }

        Ok(hash)
    }

    pub fn export(&self, str: RString) -> Result<Option<Value>, Error> {
        let store = self.store.try_convert::<&Store>()?;
        let export = self
            .inner
            .get_export(store.context_mut(), unsafe { str.as_str()? });
        match export {
            Some(export) => export.wrap_wasmtime_type(self.store).map(Some),
            None => Ok(None),
        }
    }

    pub fn invoke(&self, args: &[Value]) -> Result<Value, Error> {
        let name: RString = args
            .get(0)
            .ok_or_else(|| {
                Error::new(
                    magnus::exception::type_error(),
                    "wrong number of arguments (given 0, expected 1+)",
                )
            })?
            .try_convert()?;

        let store: &Store = self.store.try_convert()?;
        let func = self.get_func(store.context_mut(), unsafe { name.as_str()? })?;
        Func::invoke(&StoreContextValue::Store(self.store), &func, &args[1..]).map_err(|e| e.into())
    }

    fn get_func(
        &self,
        context: StoreContextMut<'_, StoreData>,
        name: &str,
    ) -> Result<wasmtime::Func, Error> {
        let instance = self.inner;

        if let Some(func) = instance.get_func(context, name) {
            Ok(func)
        } else {
            err!("function \"{}\" not found", name)
        }
    }
}

pub fn init() -> Result<(), Error> {
    let class = root().define_class("Instance", Default::default())?;

    class.define_singleton_method("new", function!(Instance::new, -1))?;
    class.define_method("invoke", method!(Instance::invoke, -1))?;
    class.define_method("exports", method!(Instance::exports, 0))?;
    class.define_method("export", method!(Instance::export, 1))?;

    Ok(())
}
