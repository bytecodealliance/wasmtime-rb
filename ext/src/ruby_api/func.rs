use super::{
    convert::{ToRubyValue, ToSym, ToValTypeVec, ToWasmVal},
    errors::result_error,
    params::Params,
    root,
    store::{Store, StoreContextValue, StoreData},
};
use crate::{helpers::WrappedStruct, Caller};
use magnus::{
    block::Proc, function, memoize, method, r_typed_data::DataTypeBuilder, scan_args::scan_args,
    DataTypeFunctions, Error, Module as _, Object, RArray, RClass, Symbol, TryConvert, TypedData,
    Value, QNIL,
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
    ///
    /// Creates a WebAssembly function from a Ruby block. WebAssembly functions
    /// can have 0 or more parameters and results. Each param and result must be a
    /// valid WebAssembly type represented as a symbol. The valid symbols are:
    /// +:i32+, +:i64+, +:f32+, +:f64+, +:v128+, +:funcref+, +:externref+.
    ///
    /// @def new(store, params, results, &block)
    /// @param store [Store]
    /// @param params [Array<Symbol>] The function's parameters.
    /// @param results [Array<Symbol>] The function's results.
    /// @param block [Block] The function's implementation.
    ///
    /// @yield [caller, *args] The function's body
    /// @yieldparam caller [Caller] Caller which can be used to interact with the {Store}.
    /// @yieldparam *args [Object] Splat of Ruby objects matching the function’s params arity.
    /// @yieldreturn [nil, Object, Array<Object>] The return type depends on function’s results arity:
    ///   * 0 => +nil+
    ///   * 1 => +Object+
    ///   * > 1 => +Array<Object>+
    ///
    /// @return [Func]
    ///
    /// @example Function that increments an i32:
    ///   store = Wasmtime::Store.new(Wasmtime::Engine.new)
    ///   Wasmtime::Func.new(store, [:i32], [:i32]) do |_caller, arg1|
    ///     arg1.succ
    ///   end
    ///
    /// @example Function with 2 params and 2 results:
    ///   store = Wasmtime::Store.new(Wasmtime::Engine.new)
    ///   Wasmtime::Func.new(store, [:i32, :i32], [:i32, :i32]) do |_caller, arg1, arg2|
    ///     [arg1.succ, arg2.succ]
    ///   end
    pub fn new(args: &[Value]) -> Result<Self, Error> {
        let args = scan_args::<(Value, RArray, RArray), (), (), (), (), Proc>(args)?;
        let (s, params, results) = args.required;
        let callable = args.block;

        let wrapped_store: WrappedStruct<Store> = s.try_convert()?;
        let store = wrapped_store.get()?;

        store.retain(callable.into());
        let context = store.context_mut();
        let ty = wasmtime::FuncType::new(params.to_val_type_vec()?, results.to_val_type_vec()?);
        let func_closure = make_func_closure(&ty, callable);
        let inner = wasmtime::Func::new(context, ty, func_closure);

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
    /// @return [nil, Object, Array<Object>] The return type depends on the function's results arity:
    ///   * 0 => +nil+
    ///   * 1 => +Object+
    ///   * > 1 => +Array<Object>+
    /// @example
    ///   store = Wasmtime::Store.new(Wasmtime::Engine.new)
    ///   func = Wasmtime::Func.new(store, [:i32, :i32], [:i32, :i32]) do |_caller, arg1, arg2|
    ///     [arg1.succ, arg2.succ]
    ///   end
    ///   func.call(1, 2) # => [2, 3]
    pub fn call(&self, args: &[Value]) -> Result<Value, Error> {
        Self::invoke(&self.store, &self.inner, args)
    }

    pub fn inner(&self) -> &FuncImpl {
        &self.inner
    }

    /// @yard
    /// @return [Array<Symbol>] The function's parameter types.
    pub fn params(&self) -> Result<Vec<Symbol>, Error> {
        let params = self
            .inner
            .ty(self.store.context()?)
            .params()
            .map(ToSym::to_sym)
            .collect();
        Ok(params)
    }

    /// @yard
    /// @return [Array<Symbol>] The function's result types.
    pub fn results(&self) -> Result<Vec<Symbol>, Error> {
        let results = self
            .inner
            .ty(self.store.context()?)
            .results()
            .map(ToSym::to_sym)
            .collect();
        Ok(results)
    }

    pub fn invoke(
        store: &StoreContextValue,
        func: &wasmtime::Func,
        args: &[Value],
    ) -> Result<Value, Error> {
        let mut context = store.context_mut()?;
        let func_ty = func.ty(&mut context);
        let params = Params::new(&func_ty, args)?.to_vec()?;
        let mut results = vec![Val::null(); func_ty.results().len()];

        func.call(context, &params, &mut results)
            .map_err(|e| store.handle_wasm_error(e))?;

        match results.as_slice() {
            [] => Ok(QNIL.into()),
            [result] => result.to_ruby_value(store),
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
            .and_then(|proc_result| {
                match results.len() {
                    0 => Ok(()), // Ignore return value
                    n => {
                        // For len=1, accept both `val` and `[val]`
                        let proc_result = RArray::try_convert(proc_result)?;
                        if proc_result.len() != n {
                            return Err(Error::new(
                                result_error(),
                                format!(
                                    "wrong number of results (given {}, expected {}) in {}",
                                    proc_result.len(),
                                    n,
                                    callable,
                                ),
                            ));
                        }
                        for (i, ((rb_val, wasm_val), ty)) in unsafe { proc_result.as_slice() }
                            .iter()
                            .zip(results.iter_mut())
                            .zip(ty.results())
                            .enumerate()
                        {
                            *wasm_val = rb_val.to_wasm_val(&ty).map_err(|e| {
                                Error::new(
                                    result_error(),
                                    format!("{} (result index {} in {})", e, i, callable),
                                )
                            })?;
                        }
                        Ok(())
                    }
                }
            })
            .map_err(|e| anyhow::anyhow!(e));

        // Drop the wasmtime::Caller so it does not outlive the Func call, if e.g. the user
        // assigned the Ruby Wasmtime::Caller instance to a global.
        caller.expire();

        result
    }
}

pub fn init() -> Result<(), Error> {
    let func = root().define_class("Func", Default::default())?;
    func.define_singleton_method("new", function!(Func::new, -1))?;
    func.define_method("call", method!(Func::call, -1))?;
    func.define_method("params", method!(Func::params, 0))?;
    func.define_method("results", method!(Func::results, 0))?;

    Ok(())
}
