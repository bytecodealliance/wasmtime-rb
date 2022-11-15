use super::{convert::WrapWasmtimeType, func::Func, memory::Memory, root, store::Store};
use crate::{conversion_err, helpers::WrappedStruct, not_implemented};
use magnus::{
    method, rb_sys::AsRawValue, DataTypeFunctions, Error, Module, RClass, TypedData, Value,
};

/// @yard
/// An external item to a WebAssembly module, or a list of what can possibly be exported from a wasm module.
/// @see https://docs.rs/wasmtime/latest/wasmtime/enum.Extern.html Wasmtime's Rust doc
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
    /// @yard
    /// @return [Wasmtime::Func] The exported function, if this is a function.
    pub fn to_func(rb_self: WrappedStruct<Self>) -> Result<Value, Error> {
        match rb_self.get()? {
            Extern::Func(f) => Ok(f.to_value()),
            _ => conversion_err!(Self::inner_class(rb_self)?, Func::class()),
        }
    }

    /// @yard
    /// @return [Wasmtime::Memory] The exported memory, if this is a memory.
    pub fn to_memory(rb_self: WrappedStruct<Self>) -> Result<Value, Error> {
        match rb_self.get()? {
            Extern::Memory(f) => Ok(f.to_value()),
            _ => conversion_err!(Self::inner_class(rb_self)?, Memory::class()),
        }
    }

    pub fn inspect(rb_self: WrappedStruct<Self>) -> Result<String, Error> {
        let rs_self = rb_self.get()?;

        let inner_string: String = match rs_self {
            Extern::Func(f) => f.inspect(),
            Extern::Memory(m) => m.inspect(),
        };

        Ok(format!(
            "#<Wasmtime::Extern:0x{:016x} @value={}>",
            rb_self.to_value().as_raw(),
            inner_string
        ))
    }

    fn inner_class(rb_self: WrappedStruct<Self>) -> Result<RClass, Error> {
        match rb_self.get()? {
            Extern::Func(f) => Ok(f.to_value().class()),
            Extern::Memory(m) => Ok(m.to_value().class()),
        }
    }
}

impl WrapWasmtimeType<Extern> for wasmtime::Extern {
    fn wrap_wasmtime_type(&self, store: WrappedStruct<Store>) -> Result<Extern, Error> {
        match self {
            wasmtime::Extern::Func(func) => Ok(Extern::Func(Func::from_inner(store, *func).into())),
            wasmtime::Extern::Memory(mem) => {
                Ok(Extern::Memory(Memory::from_inner(store, *mem).into()))
            }
            wasmtime::Extern::Global(_) => not_implemented!("global not yet supported"),
            wasmtime::Extern::Table(_) => not_implemented!("table not yet supported"),
            wasmtime::Extern::SharedMemory(_) => not_implemented!("shared memory not supported"),
        }
    }
}

pub fn init() -> Result<(), Error> {
    let class = root().define_class("Extern", Default::default())?;

    class.define_method("to_func", method!(Extern::to_func, 0))?;
    class.define_method("to_memory", method!(Extern::to_memory, 0))?;
    class.define_method("inspect", method!(Extern::inspect, 0))?;

    Ok(())
}

unsafe impl Send for Extern {}
