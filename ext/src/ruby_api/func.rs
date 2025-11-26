use super::{
    convert::{ToRubyValue, ToSym, ToValTypeVec, ToWasmVal},
    engine,
    errors::result_error,
    params::Params,
    root,
    store::{Store, StoreContextValue, StoreData},
};
use crate::{error, Caller};
use magnus::{
    block::Proc, class, function, gc::Marker, method, prelude::*, scan_args::scan_args,
    typed_data::Obj, value::Opaque, DataTypeFunctions, Error, IntoValue, Object, RArray, Ruby,
    TypedData, Value,
};
use wasmtime::{Caller as CallerImpl, Func as FuncImpl, Val};

/// @yard
/// @rename Wasmtime::FuncType
/// Represents a WebAssembly Function Type
/// @see https://docs.rs/wasmtime/latest/wasmtime/struct.FuncType.html Wasmtime's Rust doc
#[derive(TypedData)]
#[magnus(
    class = "Wasmtime::FuncType",
    size,
    mark,
    free_immediately,
    unsafe_generics
)]
pub struct FuncType {
    inner: wasmtime::FuncType,
}

impl DataTypeFunctions for FuncType {}

impl FuncType {
    pub fn from_inner(inner: wasmtime::FuncType) -> Self {
        Self { inner }
    }

    /// @yard
    /// @return [Array<Symbol>] The function's parameter types.
    pub fn params(ruby: &Ruby, rb_self: Obj<Self>) -> Result<RArray, Error> {
        let len = rb_self.inner.params().len();
        let mut params = rb_self.inner.params();
        params.try_fold(ruby.ary_new_capa(len), |array, p| {
            array.push(p.to_sym()?)?;
            Ok(array)
        })
    }

    /// @yard
    /// @return [Array<Symbol>] The function's result types.
    pub fn results(ruby: &Ruby, rb_self: Obj<Self>) -> Result<RArray, Error> {
        let len = rb_self.inner.results().len();
        let mut results = rb_self.inner.results();
        results.try_fold(ruby.ary_new_capa(len), |array, r| {
            array.push(r.to_sym()?)?;
            Ok(array)
        })
    }
}

impl From<&FuncType> for wasmtime::ExternType {
    fn from(func: &FuncType) -> Self {
        Self::Func(func.inner.clone())
    }
}

/// @yard
/// @rename Wasmtime::Func
/// Represents a WebAssembly Function
/// @see https://docs.rs/wasmtime/latest/wasmtime/struct.Func.html Wasmtime's Rust doc
#[derive(TypedData)]
#[magnus(
    class = "Wasmtime::Func",
    size,
    mark,
    free_immediately,
    unsafe_generics
)]
pub struct Func<'a> {
    store: StoreContextValue<'a>,
    inner: FuncImpl,
}

impl DataTypeFunctions for Func<'_> {
    fn mark(&self, marker: &Marker) {
        self.store.mark(marker)
    }
}

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
        let args = scan_args::<(Obj<Store>, RArray, RArray), (), (), (), (), Proc>(args)?;
        let (store, params, results) = args.required;
        let callable = args.block;

        store.retain(callable.as_value());

        let context = store.context_mut();
        let engine = context.engine();
        let ty = wasmtime::FuncType::new(
            engine,
            params.to_val_type_vec()?,
            results.to_val_type_vec()?,
        );
        let func_closure = make_func_closure(&ty, callable.into());
        let inner = wasmtime::Func::new(context, ty, func_closure);

        Ok(Self {
            store: store.into(),
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
        let ruby = Ruby::get().unwrap();
        Self::invoke(&ruby, &self.store, &self.inner, args)
    }

    pub fn inner(&self) -> &FuncImpl {
        &self.inner
    }

    /// @yard
    /// @return [Array<Symbol>] The function's parameter types.
    pub fn params(ruby: &Ruby, rb_self: Obj<Self>) -> Result<RArray, Error> {
        let ty = rb_self.inner.ty(rb_self.store.context()?);
        let len = ty.params().len();
        let mut params = ty.params();
        params.try_fold(ruby.ary_new_capa(len), |array, p| {
            array.push(p.to_sym()?)?;
            Ok(array)
        })
    }

    /// @yard
    /// @return [Array<Symbol>] The function's result types.
    pub fn results(ruby: &Ruby, rb_self: Obj<Self>) -> Result<RArray, Error> {
        let ty = rb_self.inner.ty(rb_self.store.context()?);
        let len = ty.results().len();
        let mut results = ty.results();
        results.try_fold(ruby.ary_new_capa(len), |array, r| {
            array.push(r.to_sym()?)?;
            Ok(array)
        })
    }

    pub fn invoke(
        ruby: &Ruby,
        store: &StoreContextValue,
        func: &wasmtime::Func,
        args: &[Value],
    ) -> Result<Value, Error> {
        let mut context = store.context_mut()?;
        let func_ty = func.ty(&mut context);
        let params = Params::new(ruby, &func_ty, args)?.to_vec(ruby, store)?;
        let mut results = vec![Val::null_func_ref(); func_ty.results().len()];

        func.call(context, &params, &mut results)
            .map_err(|e| store.handle_wasm_error(e))?;

        match results.as_slice() {
            [] => Ok(().into_value_with(ruby)),
            [result] => result.to_ruby_value(ruby, store),
            _ => {
                let ary = ruby.ary_new_capa(results.len());
                for result in results {
                    let val = result.to_ruby_value(ruby, store)?;
                    ary.push(val)?;
                }
                Ok(ary.into_value_with(ruby))
            }
        }
    }
}

impl From<&Func<'_>> for wasmtime::Extern {
    fn from(func: &Func) -> Self {
        Self::Func(func.get())
    }
}

macro_rules! caller_error {
    ($store:expr, $caller:expr, $error:expr) => {{
        $store.set_last_error($error);
        $caller.expire();
        Err(wasmtime::Error::msg(""))
    }};
}

macro_rules! result_error {
    ($store:expr, $caller:expr, $msg:expr) => {{
        let error = Error::new(result_error(), $msg);
        caller_error!($store, $caller, error)
    }};
}

pub fn make_func_closure(
    ty: &wasmtime::FuncType,
    callable: Opaque<Proc>,
) -> impl Fn(CallerImpl<'_, StoreData>, &[Val], &mut [Val]) -> wasmtime::Result<()> + Send + Sync + 'static
{
    let ty = ty.to_owned();

    // The error handling here is a bit tricky. We want to return a Ruby exception,
    // but doing so directly can easily cause an early Ruby GC and segfault. So to
    // be safe, we store all Ruby errors on the store context so it can be marked.
    // We then return a generic error here. The caller will check for a stored error
    // and raise it if it exists.
    move |caller_impl: CallerImpl<'_, StoreData>, params: &[Val], results: &mut [Val]| {
        let ruby = Ruby::get().unwrap();
        let wrapped_caller = ruby.obj_wrap(Caller::new(caller_impl));
        let store_context = StoreContextValue::from(wrapped_caller);

        let rparams = ruby.ary_new_capa(params.len() + 1);
        rparams
            .push(wrapped_caller.as_value())
            .map_err(|e| wasmtime::Error::msg(format!("failed to push caller: {e}")))?;

        for (i, param) in params.iter().enumerate() {
            let val = param
                .to_ruby_value(&ruby, &store_context)
                .map_err(|e| wasmtime::Error::msg(format!("invalid argument at index {i}: {e}")))?;
            rparams.push(val).map_err(|e| {
                wasmtime::Error::msg(format!("failed to push argument at index {i}: {e}"))
            })?;
        }

        let callable = ruby.get_inner(callable);

        match (callable.call(rparams), results.len()) {
            (Ok(_proc_result), 0) => {
                wrapped_caller.expire();
                Ok(())
            }
            (Ok(proc_result), n) => {
                // For len=1, accept both `val` and `[val]`
                let Ok(proc_result) = RArray::to_ary(proc_result) else {
                    return result_error!(
                        store_context,
                        wrapped_caller,
                        format!("could not convert {} to results array", callable)
                    );
                };

                if proc_result.len() != results.len() {
                    return result_error!(
                        store_context,
                        wrapped_caller,
                        format!(
                            "wrong number of results (given {}, expected {}) in {}",
                            proc_result.len(),
                            n,
                            callable
                        )
                    );
                }

                for (i, ((rb_val, wasm_val), ty)) in unsafe { proc_result.as_slice() }
                    .iter()
                    .zip(results.iter_mut())
                    .zip(ty.results())
                    .enumerate()
                {
                    match rb_val.to_wasm_val(&store_context, ty) {
                        Ok(val) => *wasm_val = val,
                        Err(e) => {
                            return result_error!(
                                store_context,
                                wrapped_caller,
                                format!("invalid result at index {i}: {e} in {callable}")
                            );
                        }
                    }
                }

                wrapped_caller.expire();
                Ok(())
            }
            (Err(e), _) => {
                caller_error!(store_context, wrapped_caller, e)
            }
        }
    }
}

pub fn init(ruby: &Ruby) -> Result<(), Error> {
    let func_type = root().define_class("FuncType", ruby.class_object())?;
    func_type.define_method("params", method!(FuncType::params, 0))?;
    func_type.define_method("results", method!(FuncType::results, 0))?;

    let func = root().define_class("Func", ruby.class_object())?;
    func.define_singleton_method("new", function!(Func::new, -1))?;
    func.define_method("call", method!(Func::call, -1))?;
    func.define_method("params", method!(Func::params, 0))?;
    func.define_method("results", method!(Func::results, 0))?;

    Ok(())
}
