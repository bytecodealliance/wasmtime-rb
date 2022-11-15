use super::{
    convert::{ToExtern, WrapWasmtimeType},
    func::Func,
    module::Module,
    root,
    store::{Store, StoreData},
};
use crate::{err, error, helpers::WrappedStruct};
use magnus::{
    function, method, scan_args, DataTypeFunctions, Error, Module as _, Object, RArray, RHash,
    RString, TypedData, Value,
};
use wasmtime::{Extern, Instance as InstanceImpl, StoreContextMut};

/// @yard
/// Represents a WebAssembly instance.
/// @see https://docs.rs/wasmtime/latest/wasmtime/struct.Instance.html Wasmtime's Rust doc
#[derive(Clone, Debug, TypedData)]
#[magnus(class = "Wasmtime::Instance", mark, free_immediatly)]
pub struct Instance {
    inner: InstanceImpl,
    store: WrappedStruct<Store>,
}

unsafe impl Send for Instance {}

impl DataTypeFunctions for Instance {
    fn mark(&self) {
        self.store.mark()
    }
}

impl Instance {
    /// @yard
    /// @def new(store, mod, imports = [])
    /// @param store [Store] The store to instantiate the module in.
    /// @param mod [Module] The module to instantiate.
    /// @param imports [Array<Func, Memory>]
    ///   The module's import, in orders that that they show up in the module.
    /// @return [Instance]
    pub fn new(args: &[Value]) -> Result<Self, Error> {
        let args =
            scan_args::scan_args::<(Value, &Module), (Option<Value>,), (), (), (), ()>(args)?;
        let (s, module) = args.required;
        let wrapped_store: WrappedStruct<Store> = s.try_convert()?;
        let store = wrapped_store.get()?;
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

        Ok(Self {
            inner,
            store: wrapped_store,
        })
    }

    pub fn get(&self) -> InstanceImpl {
        self.inner
    }

    pub fn from_inner(store: WrappedStruct<Store>, inner: InstanceImpl) -> Self {
        Self { inner, store }
    }

    /// @yard
    /// Returns a Hash of exports where keys are the export name (String).
    ///
    /// @def exports
    /// @return [Hash{String => Func, Memory}]
    pub fn exports(&self) -> Result<RHash, Error> {
        let store = self.store.get()?;
        let mut ctx = store.context_mut();
        let hash = RHash::new();

        for export in self.inner.exports(&mut ctx) {
            let export_name: RString = export.name().into();
            let wrapped_store = self.store.clone();
            let wrapped_export = export.into_extern().wrap_wasmtime_type(wrapped_store)?;
            hash.aset(export_name, wrapped_export)?;
        }

        Ok(hash)
    }

    /// @yard
    /// Get an export by name.
    ///
    /// @def export(name)
    /// @param name [String]
    /// @return [Func, Memory, nil] The export if it exists, nil otherwise.
    pub fn export(&self, str: RString) -> Result<Option<super::externals::Extern>, Error> {
        let store = self.store.get()?;
        let export = self
            .inner
            .get_export(store.context_mut(), unsafe { str.as_str()? });
        match export {
            Some(export) => export.wrap_wasmtime_type(self.store.clone()).map(Some),
            None => Ok(None),
        }
    }

    /// @yard
    /// Retrieves a Wasm function from the instance and calls it.
    /// Essentially a shortcut for +instance.export(name).call(...)+.
    ///
    /// @def invoke(name, *args)
    /// @param name [String] The name of function  to run.
    /// @param (see Func#call)
    /// @return (see Func#call)
    /// @see Func#call
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
        Func::invoke(store, &func, &args[1..]).map_err(|e| e.into())
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
