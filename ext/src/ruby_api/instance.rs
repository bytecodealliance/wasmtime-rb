use super::{
    convert::ToExtern,
    export::Export,
    func::Func,
    module::Module,
    root,
    store::{Store, StoreData},
};
use crate::{err, error};
use magnus::{
    function, gc, method, scan_args, DataTypeFunctions, Error, Module as _, Object, RArray, RHash,
    TypedData, Value,
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
                .exception()
                .take()
                .map(Error::from)
                .unwrap_or_else(|| error!("{}", e))
        })?;

        Ok(Self { inner, store: s })
    }

    pub fn exports(&self) -> Result<RHash, Error> {
        let store = self.store.try_convert::<&Store>()?;
        let mut ctx = store.context_mut();
        let hash = RHash::new();
        let exports = self
            .inner
            .exports(&mut ctx)
            .map(|export| Export::new(store, export));

        for export in exports {
            let name = export.name();
            hash.aset(name, export)?;
        }

        Ok(hash)
    }

    pub fn invoke(&self, name: String, args: RArray) -> Result<Value, Error> {
        let store: &Store = self.store.try_convert()?;
        let func = self.get_func(store.context_mut(), &name)?;
        Func::invoke(store, &func, args).map_err(|e| e.into())
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
    class.define_method("invoke", method!(Instance::invoke, 2))?;
    class.define_method("exports", method!(Instance::exports, 0))?;

    Ok(())
}
