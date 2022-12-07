use super::{
    convert::{ToRubyValue, ToWasmVal},
    func_type::FuncType,
    params::Params,
    root,
    store::{Store, StoreContextValue, StoreData},
};
use crate::{error, helpers::WrappedStruct, Caller};
use magnus::{
    block::Proc, function, memoize, method, r_typed_data::DataTypeBuilder, scan_args::scan_args,
    value::BoxValue, DataTypeFunctions, Error, Exception, Module as _, Object, RArray, RClass,
    TryConvert, TypedData, Value, QNIL,
};
use wasmtime::{Caller as CallerImpl, Func as FuncImpl, Val};

/// @yard
/// @rename Wasmtime::Func
/// Represents a WebAssembly Function
/// @see https://docs.rs/wasmtime/latest/wasmtime/struct.Func.html Wasmtime's Rust doc
#[derive(Debug)]
pub struct Func<'a> {
    store: StoreContextValue<'a>,
    inner: FuncImpl,
}

unsafe impl<'a> TypedData for Func<'a> {
    fn class() -> magnus::RClass {
        *memoize!(RClass: root().define_class("Func", Default::default()).unwrap())
    }

    fn data_type() -> &'static magnus::DataType {
        memoize!(magnus::DataType: {
            let mut builder = DataTypeBuilder::<Func<'_>>::new("Wasmtime::Func");
            builder.size();
            builder.mark();
            builder.free_immediately();
            builder.build()
        })
    }
}

impl DataTypeFunctions for Func<'_> {
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

unsafe impl Send for Func<'_> {}

impl<'a> Func<'a> {
    /// @yard
    /// @def new(store, type, callable, &block)
    /// @param store [Store]
    /// @param type [FuncType]
    /// @param block [Block] The funcs's implementation
    ///
    /// @yield [caller, *args] The function's body
    /// @yieldparam caller [Caller] Caller which can be used to interact with the {Store}.
    /// @yieldparam *args [Object] Splat of Ruby objects matching the {FuncType}’s params arity.
    /// @yieldreturn [nil, Object, Array<Object>] The return type depends on {FuncType}’s results arity:
    ///   * 0 => +nil+
    ///   * 1 => +Object+
    ///   * > 1 => +Array<Object>+
    ///
    /// @return [Func]
    ///
    /// @example Function that increments an i32:
    ///   store = Wasmtime::Store.new(Wasmtime::Engine.new)
    ///   type = FuncType.new([:i32], [:i32])
    ///   Wasmtime::Func.new(store, type) do |_caller, arg1|
    ///     arg1.succ
    ///   end
    pub fn new(args: &[Value]) -> Result<Self, Error> {
        let args = scan_args::<(Value, &FuncType), (), (), (), (), Proc>(args)?;
        let (s, functype) = args.required;
        let callable = args.block;

        let wrapped_store: WrappedStruct<Store> = s.try_convert()?;
        let store = wrapped_store.get()?;

        store.retain(callable.into());
        let context = store.context_mut();
        let ty = functype.get();

        let inner = wasmtime::Func::new(context, ty.clone(), make_func_closure(ty, callable));

        Ok(Self {
            store: wrapped_store.into(),
            inner,
        })
    }

    pub fn from_inner(store: StoreContextValue<'a>, inner: FuncImpl) -> Self {
        Self { store, inner }
    }

    pub fn get(&self) -> FuncImpl {
        // Makes a copy (wasmtime::Func implements Copy)
        self.inner
    }

    /// @yard
    /// Calls a Wasm function.
    ///
    /// @def call(*args)
    /// @param args [Object]
    ///   The arguments to send to the Wasm function. Raises if the arguments do
    ///   not conform to the Wasm function's parameters.
    ///
    /// @return [nil, Object, Array<Object>] The return type depends on {FuncType}’s results arity:
    ///   * 0 => +nil+
    ///   * 1 => +Object+
    ///   * > 1 => +Array<Object>+
    pub fn call(&self, args: &[Value]) -> Result<Value, Error> {
        Self::invoke(&self.store, &self.inner, args).map_err(|e| e.into())
    }

    pub fn inner(&self) -> &FuncImpl {
        &self.inner
    }

    pub fn invoke(
        store: &StoreContextValue,
        func: &wasmtime::Func,
        args: &[Value],
    ) -> Result<Value, InvokeError> {
        let func_ty = func.ty(store.context_mut()?);
        let param_types = func_ty.params().collect::<Vec<_>>();
        let params = Params::new(args, param_types)?.to_vec()?;
        let mut results = vec![Val::null(); func_ty.results().len()];

        func.call(store.context_mut()?, &params, &mut results)
            .map_err(|e| store.handle_wasm_error(e))?;

        match results.as_slice() {
            [] => Ok(QNIL.into()),
            [result] => result.to_ruby_value(store).map_err(|e| e.into()),
            _ => {
                let array = RArray::with_capacity(results.len());
                for result in results {
                    array.push(result.to_ruby_value(store)?)?;
                }
                Ok(array.into())
            }
        }
    }
}

impl From<&Func<'_>> for wasmtime::Extern {
    fn from(func: &Func) -> Self {
        Self::Func(func.get())
    }
}

pub fn make_func_closure(
    ty: &wasmtime::FuncType,
    callable: Proc,
) -> impl Fn(CallerImpl<'_, StoreData>, &[Val], &mut [Val]) -> anyhow::Result<()> + Send + Sync + 'static
{
    let ty = ty.to_owned();
    let callable = ShareableProc(callable);

    move |caller_impl: CallerImpl<'_, StoreData>, params: &[Val], results: &mut [Val]| {
        let wrapped_caller: WrappedStruct<Caller> = Caller::new(caller_impl).into();
        let caller = wrapped_caller.get().unwrap();
        let store_context = StoreContextValue::from(wrapped_caller);

        let rparams = RArray::with_capacity(params.len() + 1);
        rparams.push(Value::from(wrapped_caller)).unwrap();

        for (i, param) in params.iter().enumerate() {
            let rparam = param
                .to_ruby_value(&store_context)
                .map_err(|e| anyhow::anyhow!(format!("invalid argument at index {}: {}", i, e)))?;
            rparams.push(rparam).unwrap();
        }

        let callable = callable.0;

        let result = callable
            .call(unsafe { rparams.as_slice() })
            .map_err(|e| {
                if let Error::Exception(exception) = e {
                    caller.hold_exception(exception);
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
                anyhow::anyhow!(format!(
                    "Error when calling Func {}\n Error: {}",
                    callable.inspect(),
                    e
                ))
            });

        // Drop the wasmtime::Caller so it does not outlive the Func call, if e.g. the user
        // assigned the Ruby Wasmtime::Caller instance to a global.
        caller.expire();

        result
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

pub fn init() -> Result<(), Error> {
    let func = root().define_class("Func", Default::default())?;
    func.define_singleton_method("new", function!(Func::new, -1))?;
    func.define_method("call", method!(Func::call, -1))?;

    Ok(())
}
