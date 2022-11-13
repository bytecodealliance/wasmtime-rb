use lazy_static::__Deref;
use magnus::{method, DataTypeFunctions, Error, Module, RClass, TypedData, Value};

use crate::{err, helpers::WrappedStruct, not_implemented};

use super::{convert::WrapWasmtimeType, func::Func, memory::Memory, root, store::Store};

#[derive(TypedData)]
#[magnus(class = "Wasmtime::Extern", mark, free_immediatly)]
pub enum Extern {
    Func(WrappedStruct<Func>),
    Memory(WrappedStruct<Memory>),
}

impl DataTypeFunctions for Extern {
    fn mark(&self) {
        match self {
            Extern::Func(f) => f.mark(),
            Extern::Memory(m) => m.mark(),
        }
    }
}

impl Extern {
    pub fn to_func(rb_self: WrappedStruct<Self>) -> Result<Value, Error> {
        match rb_self.get()? {
            Extern::Func(f) => Ok(*f.deref()),
            _ => err!("{} is not a function", rb_self.deref().inspect()),
        }
    }

    pub fn to_memory(rb_self: WrappedStruct<Self>) -> Result<Value, Error> {
        match rb_self.get()? {
            Extern::Memory(f) => Ok(*f.deref()),
            _ => err!("{} is not a memory", rb_self.deref().inspect()),
        }
    }

    pub fn to_global(_rb_self: WrappedStruct<Self>) -> Result<Value, Error> {
        not_implemented!("Extern#to_global")
    }

    pub fn to_table(_rb_self: WrappedStruct<Self>) -> Result<Value, Error> {
        not_implemented!("Extern#to_table")
    }

    pub fn to_shared_memory(_rb_self: WrappedStruct<Self>) -> Result<Value, Error> {
        not_implemented!("Extern#to_shared_memory")
    }
}

impl WrapWasmtimeType<Extern> for wasmtime::Extern {
    fn wrap_wasmtime_type(&self, store: WrappedStruct<Store>) -> Result<Extern, Error> {
        match self {
            wasmtime::Extern::Func(func) => Ok(Extern::Func(Func::from_inner(store, *func).into())),
            wasmtime::Extern::Memory(mem) => {
                Ok(Extern::Memory(Memory::from_inner(store, *mem).into()))
            }
            wasmtime::Extern::Global(_) => err!("global not yet supported"),
            wasmtime::Extern::Table(_) => err!("table not yet supported"),
            wasmtime::Extern::SharedMemory(_) => err!("shared memory not supported"),
        }
    }
}

pub fn init() -> Result<(), Error> {
    let class = root().define_class("Extern", Default::default())?;

    class.define_method("to_func", method!(Extern::to_func, 0))?;
    class.define_method("to_memory", method!(Extern::to_memory, 0))?;
    class.define_method("to_global", method!(Extern::to_global, 0))?;
    class.define_method("to_table", method!(Extern::to_table, 0))?;
    class.define_method("to_shared_memory", method!(Extern::to_shared_memory, 0))?;

    Ok(())
}

unsafe impl Send for Extern {}
