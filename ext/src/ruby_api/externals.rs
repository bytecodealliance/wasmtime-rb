use super::{
    convert::WrapWasmtimeType, func::Func, global::Global, memory::Memory, root,
    store::StoreContextValue, table::Table,
};
use crate::{conversion_err, not_implemented};
use magnus::{
    class, gc, method, rb_sys::AsRawValue, typed_data::Obj, DataTypeFunctions, Error, Module,
    RClass, TypedData, Value,
};

/// @yard
/// @rename Wasmtime::Extern
/// An external item to a WebAssembly module, or a list of what can possibly be exported from a Wasm module.
/// @see https://docs.rs/wasmtime/latest/wasmtime/enum.Extern.html Wasmtime's Rust doc
#[derive(TypedData)]
#[magnus(
    class = "Wasmtime::Extern",
    size,
    mark,
    free_immediately,
    unsafe_generics
)]
pub enum Extern<'a> {
    Func(Obj<Func<'a>>),
    Global(Obj<Global<'a>>),
    Memory(Obj<Memory<'a>>),
    Table(Obj<Table<'a>>),
}

impl DataTypeFunctions for Extern<'_> {
    fn mark(&self) {
        match self {
            Extern::Func(f) => gc::mark(*f),
            Extern::Global(g) => gc::mark(*g),
            Extern::Memory(m) => gc::mark(*m),
            Extern::Table(t) => gc::mark(*t),
        }
    }
}
unsafe impl Send for Extern<'_> {}

impl Extern<'_> {
    /// @yard
    /// Returns the exported function or raises a `{ConversionError}` when the export is not a
    /// function.
    /// @return [Func] The exported function.
    pub fn to_func(rb_self: Obj<Self>) -> Result<Value, Error> {
        match rb_self.get() {
            Extern::Func(f) => Ok(f.as_value()),
            _ => conversion_err!(Self::inner_class(rb_self), Func::class()),
        }
    }

    /// @yard
    /// Returns the exported global or raises a `{ConversionError}` when the export is not a global.
    /// @return [Global] The exported global.
    pub fn to_global(rb_self: Obj<Self>) -> Result<Value, Error> {
        match rb_self.get() {
            Extern::Global(g) => Ok(g.as_value()),
            _ => conversion_err!(Self::inner_class(rb_self), Global::class()),
        }
    }

    /// @yard
    /// Returns the exported memory or raises a `{ConversionError}` when the export is not a
    /// memory.
    /// @return [Memory] The exported memory.
    pub fn to_memory(rb_self: Obj<Self>) -> Result<Value, Error> {
        match rb_self.get() {
            Extern::Memory(m) => Ok(m.as_value()),
            _ => conversion_err!(Self::inner_class(rb_self), Memory::class()),
        }
    }

    /// @yard
    /// Returns the exported table or raises a `{ConversionError}` when the export is not a table.
    /// @return [Table] The exported table.
    pub fn to_table(rb_self: Obj<Self>) -> Result<Value, Error> {
        match rb_self.get() {
            Extern::Table(t) => Ok(t.as_value()),
            _ => conversion_err!(Self::inner_class(rb_self), Table::class()),
        }
    }

    pub fn inspect(rb_self: Obj<Self>) -> Result<String, Error> {
        let rs_self = rb_self.get();

        let inner_string: String = match rs_self {
            Extern::Func(f) => f.inspect(),
            Extern::Global(g) => g.inspect(),
            Extern::Memory(m) => m.inspect(),
            Extern::Table(t) => t.inspect(),
        };

        Ok(format!(
            "#<Wasmtime::Extern:0x{:016x} @value={}>",
            rb_self.as_raw(),
            inner_string
        ))
    }

    fn inner_class(rb_self: Obj<Self>) -> RClass {
        match rb_self.get() {
            Extern::Func(f) => f.class(),
            Extern::Global(g) => g.class(),
            Extern::Memory(m) => m.class(),
            Extern::Table(t) => t.class(),
        }
    }
}

impl<'a> WrapWasmtimeType<'a, Extern<'a>> for wasmtime::Extern {
    fn wrap_wasmtime_type(&self, store: StoreContextValue<'a>) -> Result<Extern<'a>, Error> {
        match self {
            wasmtime::Extern::Func(func) => {
                Ok(Extern::Func(Obj::wrap(Func::from_inner(store, *func))))
            }
            wasmtime::Extern::Global(global) => Ok(Extern::Global(Obj::wrap(Global::from_inner(
                store, *global,
            )))),
            wasmtime::Extern::Memory(mem) => {
                Ok(Extern::Memory(Obj::wrap(Memory::from_inner(store, *mem)?)))
            }
            wasmtime::Extern::Table(table) => {
                Ok(Extern::Table(Obj::wrap(Table::from_inner(store, *table))))
            }
            wasmtime::Extern::SharedMemory(_) => not_implemented!("shared memory not supported"),
        }
    }
}

pub fn init() -> Result<(), Error> {
    let class = root().define_class("Extern", class::object())?;

    class.define_method("to_func", method!(Extern::to_func, 0))?;
    class.define_method("to_global", method!(Extern::to_global, 0))?;
    class.define_method("to_memory", method!(Extern::to_memory, 0))?;
    class.define_method("to_table", method!(Extern::to_table, 0))?;
    class.define_method("inspect", method!(Extern::inspect, 0))?;

    Ok(())
}
