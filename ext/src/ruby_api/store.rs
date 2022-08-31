use super::{engine::Engine, root};
use magnus::{function, gc, DataTypeFunctions, Error, Module, Object, TypedData, Value};
use std::cell::{RefCell, RefMut};
use wasmtime::Store as StoreImpl;

#[derive(TypedData, Debug)]
#[magnus(class = "Wasmtime::Store", size, free_immediatly)]
pub struct Store {
    inner: RefCell<StoreImpl<Value>>,
    data: Value,
}

impl DataTypeFunctions for Store {
    fn mark(&self) {
        gc::mark(&self.data);
    }
}

unsafe impl Send for Store {}

impl Store {
    pub fn new(engine: &Engine, data: Value) -> Self {
        let eng = engine.get();
        Self {
            inner: RefCell::new(StoreImpl::new(&eng, data)),
            data,
        }
    }

    pub fn borrow_mut(&self) -> RefMut<StoreImpl<Value>> {
        self.inner.borrow_mut()
    }
}

pub fn init() -> Result<(), Error> {
    let class = root().define_class("Store", Default::default())?;

    class.define_singleton_method("new", function!(Store::new, 2))?;

    Ok(())
}
