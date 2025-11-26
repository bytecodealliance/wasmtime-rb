use super::{
    convert::{WrapWasmtimeExternType, WrapWasmtimeType},
    func::{Func, FuncType},
    global::{Global, GlobalType},
    memory::{Memory, MemoryType},
    root,
    store::StoreContextValue,
    table::{Table, TableType},
};
use crate::{conversion_err, not_implemented};
use magnus::{
    class, gc::Marker, method, prelude::*, rb_sys::AsRawValue, typed_data::Obj, DataTypeFunctions,
    Error, Module, RClass, Ruby, TypedData, Value,
};

#[derive(TypedData)]
#[magnus(
    class = "Wasmtime::ExternType",
    size,
    mark,
    free_immediately,
    unsafe_generics
)]
pub enum ExternType {
    Func(Obj<FuncType>),
    Global(Obj<GlobalType>),
    Memory(Obj<MemoryType>),
    Table(Obj<TableType>),
}

impl DataTypeFunctions for ExternType {
    fn mark(&self, marker: &Marker) {
        match self {
            ExternType::Func(f) => marker.mark(*f),
            ExternType::Global(g) => marker.mark(*g),
            ExternType::Memory(m) => marker.mark(*m),
            ExternType::Table(t) => marker.mark(*t),
        }
    }
}
unsafe impl Send for ExternType {}

impl ExternType {
    /// @yard
    /// Returns the exported function's FuncType or raises a `{ConversionError}` when the export is not a
    /// function.
    /// @return [FuncType] The exported function's type.
    pub fn to_func_type(ruby: &Ruby, rb_self: Obj<Self>) -> Result<Value, Error> {
        match *rb_self {
            ExternType::Func(f) => Ok(f.as_value()),
            _ => conversion_err!(Self::inner_class(rb_self), Func::class(ruby)),
        }
    }

    /// @yard
    /// Returns the exported global's GlobalType or raises a `{ConversionError}` when the export is not a global.
    /// @return [GlobalType] The exported global's type.
    pub fn to_global_type(ruby: &Ruby, rb_self: Obj<Self>) -> Result<Value, Error> {
        match *rb_self {
            ExternType::Global(g) => Ok(g.as_value()),
            _ => conversion_err!(Self::inner_class(rb_self), Global::class(ruby)),
        }
    }

    /// @yard
    /// Returns the exported memory's MemoryType or raises a `{ConversionError}` when the export is not a
    /// memory.
    /// @return [MemoryType] The exported memory's type.
    pub fn to_memory_type(ruby: &Ruby, rb_self: Obj<Self>) -> Result<Value, Error> {
        match *rb_self {
            ExternType::Memory(m) => Ok(m.as_value()),
            _ => conversion_err!(Self::inner_class(rb_self), Memory::class(ruby)),
        }
    }

    /// @yard
    /// Returns the exported table's TableType or raises a `{ConversionError}` when the export is not a table.
    /// @return [TableType] The exported table's type.
    pub fn to_table_type(ruby: &Ruby, rb_self: Obj<Self>) -> Result<Value, Error> {
        match *rb_self {
            ExternType::Table(t) => Ok(t.as_value()),
            _ => conversion_err!(Self::inner_class(rb_self), Table::class(ruby)),
        }
    }

    fn inner_class(rb_self: Obj<Self>) -> RClass {
        match *rb_self {
            ExternType::Func(f) => f.class(),
            ExternType::Global(g) => g.class(),
            ExternType::Memory(m) => m.class(),
            ExternType::Table(t) => t.class(),
        }
    }
}

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
    fn mark(&self, marker: &Marker) {
        match self {
            Extern::Func(f) => marker.mark(*f),
            Extern::Global(g) => marker.mark(*g),
            Extern::Memory(m) => marker.mark(*m),
            Extern::Table(t) => marker.mark(*t),
        }
    }
}
unsafe impl Send for Extern<'_> {}

impl Extern<'_> {
    /// @yard
    /// Returns the exported function or raises a `{ConversionError}` when the export is not a
    /// function.
    /// @return [Func] The exported function.
    pub fn to_func(ruby: &Ruby, rb_self: Obj<Self>) -> Result<Value, Error> {
        match *rb_self {
            Extern::Func(f) => Ok(f.as_value()),
            _ => conversion_err!(Self::inner_class(rb_self), Func::class(ruby)),
        }
    }

    /// @yard
    /// Returns the exported global or raises a `{ConversionError}` when the export is not a global.
    /// @return [Global] The exported global.
    pub fn to_global(ruby: &Ruby, rb_self: Obj<Self>) -> Result<Value, Error> {
        match *rb_self {
            Extern::Global(g) => Ok(g.as_value()),
            _ => conversion_err!(Self::inner_class(rb_self), Global::class(ruby)),
        }
    }

    /// @yard
    /// Returns the exported memory or raises a `{ConversionError}` when the export is not a
    /// memory.
    /// @return [Memory] The exported memory.
    pub fn to_memory(ruby: &Ruby, rb_self: Obj<Self>) -> Result<Value, Error> {
        match *rb_self {
            Extern::Memory(m) => Ok(m.as_value()),
            _ => conversion_err!(Self::inner_class(rb_self), Memory::class(ruby)),
        }
    }

    /// @yard
    /// Returns the exported table or raises a `{ConversionError}` when the export is not a table.
    /// @return [Table] The exported table.
    pub fn to_table(ruby: &Ruby, rb_self: Obj<Self>) -> Result<Value, Error> {
        match *rb_self {
            Extern::Table(t) => Ok(t.as_value()),
            _ => conversion_err!(Self::inner_class(rb_self), Table::class(ruby)),
        }
    }

    pub fn inspect(rb_self: Obj<Self>) -> Result<String, Error> {
        let inner_string: String = match *rb_self {
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
        match *rb_self {
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
            wasmtime::Extern::Tag(_) => not_implemented!("exception handling not yet implemented"),
        }
    }
}

impl WrapWasmtimeExternType<ExternType> for wasmtime::ExternType {
    fn wrap_wasmtime_type(&self, ruby: &Ruby) -> Result<ExternType, Error> {
        match self {
            wasmtime::ExternType::Func(ft) => Ok(ExternType::Func(
                ruby.obj_wrap(FuncType::from_inner(ft.clone())),
            )),
            wasmtime::ExternType::Global(gt) => Ok(ExternType::Global(
                ruby.obj_wrap(GlobalType::from_inner(gt.clone())),
            )),
            wasmtime::ExternType::Memory(mt) => Ok(ExternType::Memory(
                ruby.obj_wrap(MemoryType::from_inner(mt.clone())),
            )),
            wasmtime::ExternType::Table(tt) => Ok(ExternType::Table(
                ruby.obj_wrap(TableType::from_inner(tt.clone())),
            )),
            wasmtime::ExternType::Tag(_) => {
                not_implemented!("exception handling not yet implemented")
            }
        }
    }
}

pub fn init(ruby: &Ruby) -> Result<(), Error> {
    let extern_type = root().define_class("ExternType", ruby.class_object())?;
    extern_type.define_method("to_func_type", method!(ExternType::to_func_type, 0))?;
    extern_type.define_method("to_global_type", method!(ExternType::to_global_type, 0))?;
    extern_type.define_method("to_memory_type", method!(ExternType::to_memory_type, 0))?;
    extern_type.define_method("to_table_type", method!(ExternType::to_table_type, 0))?;

    let class = root().define_class("Extern", ruby.class_object())?;

    class.define_method("to_func", method!(Extern::to_func, 0))?;
    class.define_method("to_global", method!(Extern::to_global, 0))?;
    class.define_method("to_memory", method!(Extern::to_memory, 0))?;
    class.define_method("to_table", method!(Extern::to_table, 0))?;
    class.define_method("inspect", method!(Extern::inspect, 0))?;

    Ok(())
}
