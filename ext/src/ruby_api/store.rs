use super::{engine::Engine, root};
use magnus::{function, gc, DataTypeFunctions, Error, Module, Object, TypedData, Value};
use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use wasmtime::Store as StoreImpl;

use magnus::rb_sys::raw_value;

#[derive(Debug)]
pub struct StoreData {
    user_data: Value,
    host_exception: Option<Value>,
}

#[derive(TypedData)]
#[magnus(class = "Wasmtime::Store", size, mark, free_immediatly)]
pub struct Store {
    inner: RefCell<StoreImpl<StoreData>>,
    refs: RefCell<HashMap<ValueRef, usize>>,
}

impl DataTypeFunctions for Store {
    fn mark(&self) {
        self.refs.borrow().keys().for_each(|v| v.mark());
    }
}

unsafe impl Send for Store {}
unsafe impl Send for StoreData {}

#[repr(transparent)]
struct ValueRef(Value);
impl ValueRef {
    pub fn mark(&self) {
        gc::mark(&self.0);
    }
}

impl std::hash::Hash for ValueRef {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        raw_value(self.0).hash(state)
    }
}

impl PartialEq for ValueRef {
    fn eq(&self, other: &Self) -> bool {
        raw_value(self.0) == raw_value(other.0)
    }
}

impl std::cmp::Eq for ValueRef {}

impl Store {
    pub fn new(engine: &Engine, user_data: Value) -> Self {
        let eng = engine.get();
        let store_data = StoreData {
            user_data,
            host_exception: None,
        };
        let refs = RefCell::new(HashMap::default());

        let store = Self {
            inner: RefCell::new(StoreImpl::new(eng, store_data)),
            refs,
        };
        store.remember(user_data);

        store
    }

    pub fn borrow_mut(&self) -> RefMut<StoreImpl<StoreData>> {
        self.inner.borrow_mut()
    }

    pub fn borrow(&self) -> Ref<StoreImpl<StoreData>> {
        self.inner.borrow()
    }

    // Save a reference to a Ruby object so that it does not get GC'd
    pub fn remember(&self, value: Value) {
        self.refs
            .borrow_mut()
            .entry(ValueRef(value))
            .and_modify(|count| *count += 1)
            .or_insert(1);
    }
}

pub fn init() -> Result<(), Error> {
    let class = root().define_class("Store", Default::default())?;

    class.define_singleton_method("new", function!(Store::new, 2))?;

    Ok(())
}
