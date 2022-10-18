use super::{engine::Engine, root};
use magnus::{
    exception::Exception, function, method, scan_args, value::BoxValue, DataTypeFunctions, Error,
    Module, Object, TypedData, Value, QNIL,
};
use std::cell::{RefCell, UnsafeCell};
use wasmtime::{AsContext, AsContextMut, Store as StoreImpl, StoreContext, StoreContextMut};

#[derive(Debug)]
pub struct StoreData {
    user_data: Value,
    host_exception: HostException,
}

type BoxedException = BoxValue<Exception>;
#[derive(Debug, Default)]
pub struct HostException(Option<BoxedException>);
impl HostException {
    pub fn take(&mut self) -> Option<Exception> {
        std::mem::take(&mut self.0).map(|e| e.to_owned())
    }

    pub fn hold(&mut self, e: Exception) {
        self.0 = Some(BoxValue::new(e));
    }
}

impl StoreData {
    pub fn exception(&mut self) -> &mut HostException {
        &mut self.host_exception
    }

    pub fn take_last_error(&mut self) -> Option<Error> {
        self.host_exception.take().map(Error::from)
    }

    pub fn user_data(&self) -> Value {
        self.user_data
    }
}

/// @yard
/// Represents a Store
#[derive(TypedData)]
#[magnus(class = "Wasmtime::Store", size, mark, free_immediatly)]
pub struct Store {
    inner: UnsafeCell<StoreImpl<StoreData>>,
    refs: RefCell<Vec<Value>>,
}

impl DataTypeFunctions for Store {
    fn mark(&self) {
        self.refs.borrow().iter().for_each(magnus::gc::mark);
    }
}

unsafe impl Send for Store {}
unsafe impl Send for StoreData {}

impl Store {
    /// @yard
    ///
    /// @def new(engine, data = nil)
    /// @param engine [Wasmtime::Engine]
    ///   The engine for this store.
    /// @param data [Object]
    ///   The data attached to the store. Can be retrieved through {Wasmtime::Store#data} and {Wasmtime::Caller#data}.
    /// @return [Wasmtime::Store]
    ///
    /// @example
    ///   store = Wasmtime::Store.new(Wasmtime::Engine.new)
    ///
    /// @example
    ///   store = Wasmtime::Store.new(Wasmtime::Engine.new, {})
    pub fn new(args: &[Value]) -> Result<Self, Error> {
        let args = scan_args::scan_args::<(&Engine,), (Option<Value>,), (), (), (), ()>(args)?;
        let (engine,) = args.required;
        let (user_data,) = args.optional;
        let user_data = user_data.unwrap_or_else(|| QNIL.into());

        let eng = engine.get();
        let store_data = StoreData {
            user_data,
            host_exception: HostException::default(),
        };
        let store = Self {
            inner: UnsafeCell::new(StoreImpl::new(eng, store_data)),
            refs: Default::default(),
        };

        store.retain(user_data);

        Ok(store)
    }

    /// @yard
    /// @return [Object] The passed in value in {.new}
    pub fn data(&self) -> Value {
        self.context().data().user_data()
    }

    pub fn context(&self) -> StoreContext<StoreData> {
        unsafe { (*self.inner.get()).as_context() }
    }

    pub fn context_mut(&self) -> StoreContextMut<StoreData> {
        unsafe { (*self.inner.get()).as_context_mut() }
    }

    pub fn retain(&self, value: Value) {
        self.refs.borrow_mut().push(value);
    }
}

pub fn init() -> Result<(), Error> {
    let class = root().define_class("Store", Default::default())?;

    class.define_singleton_method("new", function!(Store::new, -1))?;
    class.define_method("data", method!(Store::data, 0))?;

    Ok(())
}
