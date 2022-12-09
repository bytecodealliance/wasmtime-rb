use super::{
    convert::WrapWasmtimeType, func::Func, memory::Memory, root, store::StoreContextValue,
    table::Table,
};
use crate::{conversion_err, helpers::WrappedStruct, not_implemented};
use magnus::{
    memoize, method, r_typed_data::DataTypeBuilder, rb_sys::AsRawValue, DataTypeFunctions, Error,
    Module, RClass, TypedData, Value,
};

/// @yard
/// @rename Wasmtime::Extern
/// An external item to a WebAssembly module, or a list of what can possibly be exported from a Wasm module.
/// @see https://docs.rs/wasmtime/latest/wasmtime/enum.Extern.html Wasmtime's Rust doc
pub enum Extern<'a> {
    Func(WrappedStruct<Func<'a>>),
    Memory(WrappedStruct<Memory<'a>>),
    Table(WrappedStruct<Table<'a>>),
}

unsafe impl TypedData for Extern<'_> {
    fn class() -> magnus::RClass {
        *memoize!(RClass: root().define_class("Extern", Default::default()).unwrap())
    }

    fn data_type() -> &'static magnus::DataType {
        memoize!(magnus::DataType: {
            let mut builder = DataTypeBuilder::<Extern<'_>>::new("Wasmtime::Extern");
            builder.size();
            builder.mark();
            builder.free_immediately();
            builder.build()
        })
    }
}

impl DataTypeFunctions for Extern<'_> {
    fn mark(&self) {
        match self {
            Extern::Func(f) => f.mark(),
            Extern::Memory(m) => m.mark(),
            Extern::Table(t) => t.mark(),
        }
    }
}
unsafe impl Send for Extern<'_> {}

impl Extern<'_> {
    /// @yard
    /// Returns the exported function or raises a `{ConversionError}` when the export is not a
    /// function.
    /// @return [Func] The exported function.
    pub fn to_func(rb_self: WrappedStruct<Self>) -> Result<Value, Error> {
        match rb_self.get()? {
            Extern::Func(f) => Ok(f.to_value()),
            _ => conversion_err!(Self::inner_class(rb_self)?, Func::class()),
        }
    }

    /// @yard
    /// Returns the exported memory or raises a `{ConversionError}` when the export is not a
    /// memory.
    /// @return [Memory] The exported memory.
    pub fn to_memory(rb_self: WrappedStruct<Self>) -> Result<Value, Error> {
        match rb_self.get()? {
            Extern::Memory(m) => Ok(m.to_value()),
            _ => conversion_err!(Self::inner_class(rb_self)?, Memory::class()),
        }
    }

    /// @yard
    /// Returns the exported table or raises a `{ConversionError}` when the export is not a table.
    /// @return [Table] The exported table.
    pub fn to_table(rb_self: WrappedStruct<Self>) -> Result<Value, Error> {
        match rb_self.get()? {
            Extern::Table(t) => Ok(t.to_value()),
            _ => conversion_err!(Self::inner_class(rb_self)?, Table::class()),
        }
    }

    pub fn inspect(rb_self: WrappedStruct<Self>) -> Result<String, Error> {
        let rs_self = rb_self.get()?;

        let inner_string: String = match rs_self {
            Extern::Func(f) => f.inspect(),
            Extern::Memory(m) => m.inspect(),
            Extern::Table(t) => t.inspect(),
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
            Extern::Table(t) => Ok(t.to_value().class()),
        }
    }
}

impl<'a> WrapWasmtimeType<'a, Extern<'a>> for wasmtime::Extern {
    fn wrap_wasmtime_type(&self, store: StoreContextValue<'a>) -> Result<Extern<'a>, Error> {
        match self {
            wasmtime::Extern::Func(func) => Ok(Extern::Func(Func::from_inner(store, *func).into())),
            wasmtime::Extern::Memory(mem) => {
                Ok(Extern::Memory(Memory::from_inner(store, *mem).into()))
            }
            wasmtime::Extern::Global(_) => not_implemented!("global not yet supported"),
            wasmtime::Extern::Table(table) => {
                Ok(Extern::Table(Table::from_inner(store, *table).into()))
            }
            wasmtime::Extern::SharedMemory(_) => not_implemented!("shared memory not supported"),
        }
    }
}

pub fn init() -> Result<(), Error> {
    let class = root().define_class("Extern", Default::default())?;

    class.define_method("to_func", method!(Extern::to_func, 0))?;
    class.define_method("to_memory", method!(Extern::to_memory, 0))?;
    class.define_method("to_table", method!(Extern::to_table, 0))?;
    class.define_method("inspect", method!(Extern::inspect, 0))?;

    Ok(())
}
