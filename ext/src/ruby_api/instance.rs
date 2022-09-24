use super::{
    convert::ToRubyValue,
    export::Export,
    func::Func,
    module::Module,
    params::Params,
    root,
    store::{Store, StoreData},
};
use crate::{err, error};
use magnus::{
    function, gc, method, scan_args, DataTypeFunctions, Error, Module as _, Object, RArray, RHash,
    TypedData, Value, QNIL,
};
use wasmtime::{AsContextMut, Extern, Instance as InstanceImpl, StoreContextMut, Val};

#[derive(Clone, Debug, TypedData)]
#[magnus(class = "Wasmtime::Instance", mark)]
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
        let imports = args
            .optional
            .0
            .and_then(|v| if v.is_nil() { None } else { Some(v) });

        let imports: Vec<Extern> = match imports {
            Some(arr) => {
                let arr: RArray = arr.try_convert()?;
                let mut imports = vec![];
                for import in arr.each() {
                    let import = import?;
                    store.remember(import);
                    let func = import.try_convert::<&Func>()?;
                    imports.push(func.into());
                }
                imports
            }
            None => vec![],
        };

        let module = module.get();
        let mut store = store.borrow_mut();
        let context = store.as_context_mut();
        let inner = InstanceImpl::new(context, module, &imports).map_err(|e| error!("{}", e))?;

        Ok(Self { inner, store: s })
    }

    pub fn exports(&self) -> Result<RHash, Error> {
        let store = self.store.try_convert::<&Store>()?;
        let mut borrowed_store = store.borrow_mut();
        let mut ctx = borrowed_store.as_context_mut();
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
        let mut store = store.borrow_mut();
        let func = self.get_func(store.as_context_mut(), &name)?;
        let param_types = func.ty(store.as_context_mut()).params().collect::<Vec<_>>();
        let params_slice = unsafe { args.as_slice() };
        let params = Params::new(params_slice, param_types)?.to_vec()?;

        let results_len = func.ty(store.as_context_mut()).results().len();
        let mut results = vec![Val::null(); results_len];
        let ctx = store.as_context_mut();

        Self::invoke_func(ctx, &func, &params, results.as_mut_slice())
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

    pub fn invoke_func(
        context: StoreContextMut<'_, StoreData>,
        func: &wasmtime::Func,
        params: &[Val],
        results: &mut [Val],
    ) -> Result<Value, Error> {
        func.call(context, params, results)
            .map_err(|e| error!("Could not invoke function: {}", e))?;

        match results {
            [] => Ok(QNIL.into()),
            [result] => result.to_ruby_value(),
            _ => {
                let array = RArray::with_capacity(results.len());
                for result in results {
                    array.push(result.to_ruby_value()?)?;
                }
                Ok(array.into())
            }
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
