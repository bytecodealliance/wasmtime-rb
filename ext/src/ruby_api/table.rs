use super::{
    convert::{ToRubyValue, ToWasmVal},
    root,
    store::{Store, StoreContextValue},
    table_type::TableType,
};
use crate::{error, helpers::WrappedStruct};
use magnus::{
    function, memoize, method, r_typed_data::DataTypeBuilder, DataTypeFunctions, Error,
    Module as _, Object, RClass, TypedData, Value, QNIL,
};
use wasmtime::{Extern, Table as TableImpl};

/// @yard
/// @rename Wasmtime::Table
/// Represents a WebAssembly table.
/// @see https://docs.rs/wasmtime/latest/wasmtime/struct.Table.html Wasmtime's Rust doc
#[derive(Debug)]
pub struct Table<'a> {
    store: StoreContextValue<'a>,
    inner: TableImpl,
}

unsafe impl TypedData for Table<'_> {
    fn class() -> magnus::RClass {
        *memoize!(RClass: root().define_class("Table", Default::default()).unwrap())
    }

    fn data_type() -> &'static magnus::DataType {
        memoize!(magnus::DataType: {
            let mut builder = DataTypeBuilder::<Table<'_>>::new("Wasmtime::Table");
            builder.free_immediately();
            builder.mark();
            builder.build()
        })
    }
}

impl DataTypeFunctions for Table<'_> {
    fn mark(&self) {
        self.store.mark()
    }
}

impl<'a> Table<'a> {
    /// @yard
    /// @def new(store, tabletype, initial)
    /// @param store [Store]
    /// @param tabletype [TableType]
    /// @param initial [Value] The initial value for values in the table.
    pub fn new(
        s: WrappedStruct<Store>,
        tabletype: &TableType,
        default: Value,
    ) -> Result<Self, Error> {
        let store = s.get()?;
        let default_val = default.to_wasm_val(&tabletype.get().element())?;

        let inner = TableImpl::new(store.context_mut(), tabletype.get().clone(), default_val)
            .map_err(|e| error!("{}", e))?;

        let table = Self {
            store: s.into(),
            inner,
        };

        table.retain_non_nil_extern_ref(default)?;

        Ok(table)
    }

    pub fn from_inner(store: StoreContextValue<'a>, inner: TableImpl) -> Self {
        Self { store, inner }
    }

    /// @yard
    /// Returns the table element value at +index+, or +nil+ if index is out of bound.
    ///
    /// @def get(index)
    /// @param index [Integer]
    /// @return [Object, nil]
    pub fn get(&self, index: u32) -> Result<Value, Error> {
        match self.inner.get(self.store.context_mut()?, index) {
            Some(wasm_val) => wasm_val.to_ruby_value(&self.store),
            None => Ok(*QNIL),
        }
    }

    /// @yard
    /// Sets the table entry at +index+ to +value+.
    ///
    /// @def set(index, value)
    /// @param index [Integer]
    /// @param value [Object]
    /// @return [void]
    pub fn set(&self, index: u32, value: Value) -> Result<(), Error> {
        self.inner
            .set(
                self.store.context_mut()?,
                index,
                value.to_wasm_val(&self.value_type()?)?,
            )
            .map_err(|e| error!("{}", e))
            .and_then(|result| {
                self.retain_non_nil_extern_ref(value)?;
                Ok(result)
            })
    }

    /// @yard
    /// Grows the size of this table by +delta+.
    /// Raises if the table grows beyond its limit.
    ///
    /// @def grow(delta, initial)
    /// @param delta [Integer] The number of elements to add to the table.
    /// @param initial [Object] The initial value for newly added table slots.
    /// @return [void]
    pub fn grow(&self, delta: u32, initial: Value) -> Result<u32, Error> {
        self.inner
            .grow(
                self.store.context_mut()?,
                delta,
                initial.to_wasm_val(&self.value_type()?)?,
            )
            .map_err(|e| error!("{}", e))
            .and_then(|result| {
                self.retain_non_nil_extern_ref(initial)?;
                Ok(result)
            })
    }

    /// @yard
    /// @return [Integer] The size of the table.
    pub fn size(&self) -> Result<u32, Error> {
        Ok(self.inner.size(self.store.context()?))
    }

    /// @yard
    /// @return [TableType]
    pub fn ty(&self) -> Result<TableType, Error> {
        Ok(self.inner.ty(self.store.context()?).into())
    }

    fn value_type(&self) -> Result<wasmtime::ValType, Error> {
        Ok(self.inner.ty(self.store.context()?).element())
    }

    fn retain_non_nil_extern_ref(&self, value: Value) -> Result<(), Error> {
        if wasmtime::ValType::ExternRef == self.value_type()? && !value.is_nil() {
            self.store.retain(value)?;
        }
        Ok(())
    }

    pub fn inner(&self) -> TableImpl {
        self.inner
    }
}

impl From<&Table<'_>> for Extern {
    fn from(table: &Table) -> Self {
        Self::Table(table.inner())
    }
}

pub fn init() -> Result<(), Error> {
    let class = root().define_class("Table", Default::default())?;
    class.define_singleton_method("new", function!(Table::new, 3))?;
    class.define_method("get", method!(Table::get, 1))?;
    class.define_method("set", method!(Table::set, 2))?;
    class.define_method("grow", method!(Table::grow, 2))?;
    class.define_method("size", method!(Table::size, 0))?;
    class.define_method("ty", method!(Table::ty, 0))?;

    Ok(())
}
