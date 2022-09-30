use super::{engine::Engine, root};
use magnus::{
    exception::Exception, function, value::BoxValue, DataTypeFunctions, Error, Module, Object,
    TypedData, Value,
};
use std::cell::{Ref, RefCell, RefMut};
use wasmtime::Store as StoreImpl;

#[derive(Debug)]
pub struct StoreData {
    user_data: Value,
    host_exception: HostException,
    refs: Vec<BoxValue<Value>>,
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
    pub fn root_value(&mut self, value: Value) {
        self.refs.push(BoxValue::new(value));
    }
}

#[derive(TypedData)]
#[magnus(class = "Wasmtime::Store", size, free_immediatly)]
pub struct Store {
    inner: RefCell<StoreImpl<StoreData>>,
}

impl DataTypeFunctions for Store {}

unsafe impl Send for Store {}
unsafe impl Send for StoreData {}

impl Store {
    pub fn new(engine: &Engine, user_data: Value) -> Self {
        let eng = engine.get();
        let mut store_data = StoreData {
            user_data,
            refs: Default::default(),
            host_exception: HostException::default(),
        };
        store_data.root_value(user_data);
        Self {
            inner: RefCell::new(StoreImpl::new(eng, store_data)),
        }
    }

    pub fn borrow_mut(&self) -> RefMut<StoreImpl<StoreData>> {
        self.inner.borrow_mut()
    }

    pub fn borrow(&self) -> Ref<StoreImpl<StoreData>> {
        self.inner.borrow()
    }
}

pub fn init() -> Result<(), Error> {
    let class = root().define_class("Store", Default::default())?;

    class.define_singleton_method("new", function!(Store::new, 2))?;

    Ok(())
}
