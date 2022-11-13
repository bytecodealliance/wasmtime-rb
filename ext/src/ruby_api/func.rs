use super::{
    convert::{ToRubyValue, ToWasmVal},
    func_type::FuncType,
    params::Params,
    root,
    store::{Store, StoreData},
};
use crate::{error, helpers::WrappedStruct};
use magnus::{
    block::Proc, function, memoize, method, r_typed_data::DataTypeBuilder,
    scan_args::scan_args, value::BoxValue, DataTypeFunctions, Error, Exception, Module as _,
    Object, RArray, RClass, RHash, RString, TryConvert, TypedData, Value, QNIL,
};
use std::cell::RefCell;
use wasmtime::{
    AsContextMut, Caller as CallerImpl, Extern, ExternType, Func as FuncImpl, Trap, Val,
};

#[derive(TypedData, Debug)]
#[magnus(class = "Wasmtime::Func", mark, size, free_immediatly)]
pub struct Func {
    store: WrappedStruct<Store>,
    inner: FuncImpl,
}

impl DataTypeFunctions for Func {
    fn mark(&self) {
        self.store.mark()
    }
}

// Wraps a Proc to satisfy wasmtime::Func's Send+Sync requirements. This is safe
// to do as long as (1) we hold the GVL when whe execute the proc and (2) we do
// not have multiple threads running at once (e.g. with Wasm thread proposal).
#[repr(transparent)]
struct ShareableProc(Proc);
unsafe impl Send for ShareableProc {}
unsafe impl Sync for ShareableProc {}

unsafe impl Send for Func {}

impl Func {
    pub fn new(args: &[Value]) -> Result<Self, Error> {
        let args = scan_args::<(Value, &FuncType), (), (), (), RHash, Proc>(args)?;
        let (s, functype) = args.required;
        let callable = args.block;

        let wrapped_store: WrappedStruct<Store> = s.try_convert()?;
        let store = wrapped_store.get()?;

        store.retain(callable.into());
        let context = store.context_mut();
        let ty = functype.get();

        let inner = wasmtime::Func::new(context, ty.clone(), make_func_closure(ty, callable));

        Ok(Self {
            store: wrapped_store,
            inner,
        })
    }

    pub fn from_inner(store: WrappedStruct<Store>, inner: FuncImpl) -> Self {
        Self { store, inner }
    }

    pub fn get(&self) -> FuncImpl {
        // Makes a copy (wasmtime::Func implements Copy)
        self.inner
    }

    pub fn call(&self, args: &[Value]) -> Result<Value, Error> {
        let store: &Store = self.store.try_convert()?;
        Self::invoke(store, &self.inner, args).map_err(|e| e.into())
    }

    pub fn invoke(
        store: &Store,
        func: &wasmtime::Func,
        args: &[Value],
    ) -> Result<Value, InvokeError> {
        let func_ty = func.ty(store.context_mut());
        let param_types = func_ty.params().collect::<Vec<_>>();
        let params = Params::new(args, param_types)?.to_vec()?;
        let mut results = vec![Val::null(); func_ty.results().len()];

        func.call(store.context_mut(), &params, &mut results)
            .map_err(|e| {
                store
                    .context_mut()
                    .data_mut()
                    .take_last_error()
                    .unwrap_or_else(|| error!("Could not invoke function: {}", e))
            })?;

        match results.as_slice() {
            [] => Ok(QNIL.into()),
            [result] => result.to_ruby_value().map_err(|e| e.into()),
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

impl From<&Func> for Extern {
    fn from(func: &Func) -> Self {
        Self::Func(func.get())
    }
}

pub fn make_func_closure(
    ty: &wasmtime::FuncType,
    callable: Proc,
) -> impl Fn(CallerImpl<'_, StoreData>, &[Val], &mut [Val]) -> Result<(), Trap> + Send + Sync + 'static
{
    let ty = ty.to_owned();
    let callable = ShareableProc(callable);

    move |caller: CallerImpl<'_, StoreData>, params: &[Val], results: &mut [Val]| {
        let caller = RefCell::new(caller);

        let rparams = RArray::with_capacity(params.len() + 1);
        rparams.push(Caller { inner: &caller }).ok();

        for (i, param) in params.iter().enumerate() {
            let rparam = param.to_ruby_value().map_err(|e| {
                wasmtime::Trap::new(format!("invalid argument at index {}: {}", i, e))
            })?;
            rparams.push(rparam).ok();
        }

        let callable = callable.0;
        callable
            .call(unsafe { rparams.as_slice() })
            .map_err(|e| {
                if let Error::Exception(exception) = e {
                    caller.borrow_mut().data_mut().exception().hold(exception);
                }
                e
            })
            .and_then(|proc_result| {
                match results.len() {
                    0 => Ok(()), // Ignore return value
                    n => {
                        // For len=1, accept both `val` and `[val]`
                        let proc_result = RArray::try_convert(proc_result)?;
                        if proc_result.len() != n {
                            return Result::Err(error!(
                                "wrong number of results (given {}, expected {})",
                                proc_result.len(),
                                n
                            ));
                        }
                        for ((rb_val, wasm_val), ty) in unsafe { proc_result.as_slice() }
                            .iter()
                            .zip(results.iter_mut())
                            .zip(ty.results())
                        {
                            *wasm_val = rb_val.to_wasm_val(&ty)?;
                        }
                        Ok(())
                    }
                }
            })
            .map_err(|e| {
                wasmtime::Trap::new(format!(
                    "Error when calling Func {}\n Error: {}",
                    callable.inspect(),
                    e
                ))
            })
    }
}

pub enum InvokeError {
    BoxedException(BoxValue<Exception>),
    Error(Error),
}

impl From<InvokeError> for magnus::Error {
    fn from(e: InvokeError) -> Self {
        match e {
            InvokeError::Error(e) => e,
            InvokeError::BoxedException(e) => Error::from(e.to_owned()),
        }
    }
}

impl From<magnus::Error> for InvokeError {
    fn from(e: magnus::Error) -> Self {
        InvokeError::Error(e)
    }
}

impl From<BoxValue<Exception>> for InvokeError {
    fn from(e: BoxValue<Exception>) -> Self {
        InvokeError::BoxedException(e)
    }
}

struct Caller<'a> {
    inner: &'a RefCell<CallerImpl<'a, StoreData>>,
}

impl<'a> Caller<'a> {
    pub fn store_data(&self) -> Value {
        self.inner.borrow().data().user_data()
    }

    pub fn export(&self, name: RString) -> Result<Option<Value>, Error> {
        let mut caller_mut = self.inner.borrow_mut();

        let ext = caller_mut
            .get_export(unsafe { name.as_str() }?)
            .map(|export| match export.ty(caller_mut.as_context_mut()) {
                ExternType::Func(_func) => {
                    todo!("Handle externs")
                }
                ExternType::Memory(_mem) => {
                    todo!("Handle externs")
                }
                ExternType::Table(_table) => {
                    todo!("Handle externs")
                }
                ExternType::Global(_global) => {
                    todo!("Handle externs")
                }
            });

        Ok(ext)
    }
}

unsafe impl<'a> TypedData for Caller<'a> {
    fn class() -> magnus::RClass {
        *memoize!(RClass: root().define_class("Caller", Default::default()).unwrap())
    }

    fn data_type() -> &'static magnus::DataType {
        memoize!(magnus::DataType: {
            let mut builder = DataTypeBuilder::<Caller<'_>>::new("Wasmtime::Caller");
            builder.free_immediatly();
            builder.build()
        })
    }
}
impl DataTypeFunctions for Caller<'_> {}
unsafe impl Send for Caller<'_> {}

pub fn init() -> Result<(), Error> {
    let func = root().define_class("Func", Default::default())?;
    func.define_singleton_method("new", function!(Func::new, -1))?;
    func.define_method("call", method!(Func::call, -1))?;

    let caller = root().define_class("Caller", Default::default())?;
    caller.define_method("store_data", method!(Caller::store_data, 0))?;
    caller.define_method("export", method!(Caller::export, 1))?;

    Ok(())
}
