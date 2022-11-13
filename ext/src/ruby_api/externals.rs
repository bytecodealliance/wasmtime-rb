use super::{convert::WrapWasmtimeType, func::Func, memory::Memory, root, store::Store};
use crate::{err, helpers::WrappedStruct, not_implemented};
use magnus::{method, rb_sys::raw_value, DataTypeFunctions, Error, Module, TypedData, Value};

#[derive(TypedData)]
#[magnus(class = "Wasmtime::Extern", size, mark, free_immediatly)]
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
            Extern::Func(f) => Ok(f.to_value()),
            _ => err!("{} is not a function", rb_self.to_value().inspect()),
        }
    }

    pub fn to_memory(rb_self: WrappedStruct<Self>) -> Result<Value, Error> {
        match rb_self.get()? {
            Extern::Memory(f) => Ok(f.to_value()),
            _ => err!("{} is not a memory", rb_self.to_value().inspect()),
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

    pub fn inspect(rb_self: WrappedStruct<Self>) -> Result<String, Error> {
        let rs_self = rb_self.get()?;

        let inner_string: String = match rs_self {
            Extern::Func(f) => f.inspect(),
            Extern::Memory(m) => m.inspect(),
        };

        Ok(format!(
            "#<Wasmtime::Extern:0x{:016x} @value={}>",
            raw_value(rb_self.to_value()),
            inner_string
        ))
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
    class.define_method("inspect", method!(Extern::inspect, 0))?;

    Ok(())
}

unsafe impl Send for Extern {}
