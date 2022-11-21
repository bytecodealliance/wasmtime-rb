use super::{
    convert::{ToRubyValue, ToWasmVal, WrapWasmtimeType},
    externals::Extern,
    func_type::FuncType,
    params::Params,
    root,
    store::{Store, StoreContextValue, StoreData},
};
use crate::{error, helpers::WrappedStruct};
use magnus::{
    block::Proc, function, memoize, method, r_typed_data::DataTypeBuilder, scan_args::scan_args,
    value::BoxValue, DataTypeFunctions, Error, Exception, Module as _, Object, RArray, RClass,
    RString, TryConvert, TypedData, Value, QNIL,
};
use std::cell::UnsafeCell;
use wasmtime::{
    AsContext, AsContextMut, Caller as CallerImpl, Func as FuncImpl, StoreContext, StoreContextMut,
    Trap, Val,
};

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

impl From<&Func<'_>> for wasmtime::Extern {
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

    move |caller_impl: CallerImpl<'_, StoreData>, params: &[Val], results: &mut [Val]| {
        let caller_value: Value = Caller::new(caller_impl).into();
        let caller = caller_value.try_convert::<&Caller>().unwrap();

        let rparams = RArray::with_capacity(params.len() + 1);
        rparams.push(caller_value).unwrap();

        for (i, param) in params.iter().enumerate() {
            let rparam = param.to_ruby_value().map_err(|e| {
                wasmtime::Trap::new(format!("invalid argument at index {}: {}", i, e))
            })?;
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
                wasmtime::Trap::new(format!(
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

/// A handle to a [`wasmtime::Caller`] that's only valid during a Func execution.
/// [`UnsafeCell`] wraps the wasmtime::Caller because the Value's lifetime can't
/// be tied to the Caller: the Value is handed back to Ruby and we can't control
/// whether the user keeps a handle to it or not.
#[derive(Debug)]
pub struct CallerHandle<'a> {
    caller: UnsafeCell<Option<CallerImpl<'a, StoreData>>>,
}

impl<'a> CallerHandle<'a> {
    pub fn new(caller: CallerImpl<'a, StoreData>) -> Self {
        Self {
            caller: UnsafeCell::new(Some(caller)),
        }
    }

    pub fn get_mut(&self) -> Result<&mut CallerImpl<'a, StoreData>, Error> {
        unsafe { &mut *self.caller.get() }
            .as_mut()
            .ok_or_else(|| error!("Caller outlived its Func execution"))
    }

    pub fn get(&self) -> Result<&CallerImpl<'a, StoreData>, Error> {
        unsafe { (*self.caller.get()).as_ref() }
            .ok_or_else(|| error!("Caller outlived its Func execution"))
    }

    pub fn expire(&self) {
        unsafe { *self.caller.get() = None }
    }
}

/// @yard
/// @rename Wasmtime::Caller
/// Represents the Caller's context within a Func execution. An instance of
/// Caller is sent as the first parameter to Func's implementation (the
/// block argument in {Func.new}).
/// @see https://docs.rs/wasmtime/latest/wasmtime/struct.Caller.html Wasmtime's Rust doc
#[derive(Debug)]
pub struct Caller<'a> {
    handle: CallerHandle<'a>,
}

impl<'a> Caller<'a> {
    pub fn new(caller: CallerImpl<'a, StoreData>) -> Self {
        Self {
            handle: CallerHandle::new(caller),
        }
    }

    /// @yard
    /// Returns the store's data. Akin to {Store#data}.
    /// @return [Object] The store's data (the object passed to {Store.new}).
    pub fn store_data(&self) -> Result<Value, Error> {
        self.context().map(|ctx| ctx.data().user_data())
    }

    /// @yard
    /// @def export(name)
    /// @see Instance#export
    pub fn export(
        rb_self: WrappedStruct<Caller<'a>>,
        name: RString,
    ) -> Result<Option<Extern<'a>>, Error> {
        let caller = rb_self.try_convert::<&Self>()?;
        let inner = caller.handle.get_mut()?;

        if let Some(export) = inner.get_export(unsafe { name.as_str() }?) {
            export.wrap_wasmtime_type(rb_self.into()).map(Some)
        } else {
            Ok(None)
        }
    }

    pub fn context(&self) -> Result<StoreContext<StoreData>, Error> {
        self.handle.get().map(|c| c.as_context())
    }

    pub fn context_mut(&self) -> Result<StoreContextMut<StoreData>, Error> {
        self.handle.get_mut().map(|c| c.as_context_mut())
    }

    pub fn expire(&self) {
        self.handle.expire();
    }

    fn hold_exception(&self, exception: Exception) {
        self.context_mut()
            .unwrap()
            .data_mut()
            .exception()
            .hold(exception);
    }
}

unsafe impl<'a> TypedData for Caller<'a> {
    fn class() -> magnus::RClass {
        *memoize!(RClass: root().define_class("Caller", Default::default()).unwrap())
    }

    fn data_type() -> &'static magnus::DataType {
        memoize!(magnus::DataType: {
            let mut builder = DataTypeBuilder::<Caller<'_>>::new("Wasmtime::Caller");
            builder.free_immediately();
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
