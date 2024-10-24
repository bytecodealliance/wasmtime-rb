use super::{
    convert::{ToRubyValue, ToSym, ToValType, ToWasmVal},
    root,
    store::{Store, StoreContextValue},
};

use anyhow::anyhow;

use crate::{define_rb_intern, error};
use magnus::{
    class, function, gc::Marker, method, prelude::*, scan_args, typed_data::Obj, DataTypeFunctions,
    Error, IntoValue, Object, Symbol, TypedData, Value,
};
use wasmtime::{Extern, Table as TableImpl, Val};

define_rb_intern!(
    MIN_SIZE => "min_size",
    MAX_SIZE => "max_size",
);

#[derive(TypedData)]
#[magnus(class = "Wasmtime::TableType", free_immediately, mark, unsafe_generics)]
pub struct TableType {
    inner: wasmtime::TableType,
}

impl DataTypeFunctions for TableType {
    fn mark(&self, _marker: &Marker) {}
}

impl TableType {
    pub fn from_inner(inner: wasmtime::TableType) -> Self {
        Self { inner }
    }

    /// @yard
    /// @def type
    /// @return [Symbol] The Wasm type of the elements of this table.
    pub fn type_(&self) -> Result<Symbol, Error> {
        self.inner.element().to_sym()
    }

    /// @yard
    /// @return [Integer] The minimum size of this table.
    pub fn min_size(&self) -> u64 {
        self.inner.minimum()
    }

    /// @yard
    /// @return [Integer, nil] The maximum size of this table.
    pub fn max_size(&self) -> Option<u64> {
        self.inner.maximum()
    }
}

impl From<&TableType> for wasmtime::ExternType {
    fn from(table_type: &TableType) -> Self {
        Self::Table(table_type.inner.clone())
    }
}

/// @yard
/// @rename Wasmtime::Table
/// Represents a WebAssembly table.
/// @see https://docs.rs/wasmtime/latest/wasmtime/struct.Table.html Wasmtime's Rust doc
#[derive(TypedData)]
#[magnus(class = "Wasmtime::Table", free_immediately, mark, unsafe_generics)]
pub struct Table<'a> {
    store: StoreContextValue<'a>,
    inner: TableImpl,
}

impl DataTypeFunctions for Table<'_> {
    fn mark(&self, marker: &Marker) {
        self.store.mark(marker)
    }
}

impl<'a> Table<'a> {
    /// @yard
    /// @def new(store, type, initial, min_size:, max_size: nil)
    /// @param store [Store]
    /// @param type [Symbol] The WebAssembly type of the value held by this table.
    /// @param initial [Value] The initial value of values in the table.
    /// @param min_size [Integer] The minimum number of elements in the table.
    /// @param max_size [Integer, nil] The maximum number of elements in the table.
    pub fn new(args: &[Value]) -> Result<Self, Error> {
        let args = scan_args::scan_args::<(Obj<Store>, Symbol, Value), (), (), (), _, ()>(args)?;
        let kw = scan_args::get_kwargs::<_, (u32,), (Option<u32>,), ()>(
            args.keywords,
            &[*MIN_SIZE],
            &[*MAX_SIZE],
        )?;
        let (store, value_type, default) = args.required;
        let (min,) = kw.required;
        let (max,) = kw.optional;
        let wasm_type = value_type.to_val_type()?;
        let wasm_default = default.to_wasm_val(&store.into(), wasm_type.clone())?;
        let ref_ = wasm_default
            .ref_()
            .ok_or_else(|| error!("Expected Ref for table value"))?;

        let table_type = wasm_type
            .as_ref()
            .ok_or_else(|| error!("Expected RefType"))?
            .clone();

        let inner = TableImpl::new(
            store.context_mut(),
            wasmtime::TableType::new(table_type, min, max),
            ref_,
        )
        .map_err(|e| error!("{}", e))?;

        let table = Self {
            store: store.into(),
            inner,
        };

        table.retain_non_nil_extern_ref(default)?;

        Ok(table)
    }

    pub fn from_inner(store: StoreContextValue<'a>, inner: TableImpl) -> Self {
        Self { store, inner }
    }

    /// @yard
    /// @def type
    /// @return [Symbol] The Wasm type of the elements of this table.
    pub fn type_(&self) -> Result<Symbol, Error> {
        self.ty()?.element().to_sym()
    }

    /// @yard
    /// @return [Integer] The minimum size of this table.
    pub fn min_size(&self) -> Result<u64, Error> {
        self.ty().map(|ty| ty.minimum())
    }

    /// @yard
    /// @return [Integer, nil] The maximum size of this table.
    pub fn max_size(&self) -> Result<Option<u64>, Error> {
        self.ty().map(|ty| ty.maximum())
    }

    /// @yard
    /// Returns the table element value at +index+, or +nil+ if index is out of bound.
    ///
    /// @def get(index)
    /// @param index [Integer]
    /// @return [Object, nil]
    pub fn get(&self, index: u64) -> Result<Value, Error> {
        match self.inner.get(self.store.context_mut()?, index) {
            Some(wasm_val) => Val::from(wasm_val).to_ruby_value(&self.store),
            None => Ok(().into_value()),
        }
    }

    /// @yard
    /// Sets the table entry at +index+ to +value+.
    ///
    /// @def set(index, value)
    /// @param index [Integer]
    /// @param value [Object]
    /// @return [void]
    pub fn set(&self, index: u64, value: Value) -> Result<(), Error> {
        self.inner
            .set(
                self.store.context_mut()?,
                index,
                value
                    .to_wasm_val(&self.store, wasmtime::ValType::from(self.value_type()?))?
                    .ref_()
                    .ok_or_else(|| error!("Expected Ref"))?,
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
    pub fn grow(&self, delta: u64, initial: Value) -> Result<u64, Error> {
        self.inner
            .grow(
                self.store.context_mut()?,
                delta,
                initial
                    .to_wasm_val(&self.store, wasmtime::ValType::from(self.value_type()?))?
                    .ref_()
                    .ok_or_else(|| error!("Expected Ref"))?,
            )
            .map_err(|e| error!("{}", e))
            .and_then(|result| {
                self.retain_non_nil_extern_ref(initial)?;
                Ok(result)
            })
    }

    /// @yard
    /// @return [Integer] The size of the table.
    pub fn size(&self) -> Result<u64, Error> {
        Ok(self.inner.size(self.store.context()?))
    }

    fn ty(&self) -> Result<wasmtime::TableType, Error> {
        Ok(self.inner.ty(self.store.context()?))
    }

    fn value_type(&self) -> Result<wasmtime::RefType, Error> {
        let table_type = self.inner.ty(self.store.context()?);
        let el = table_type.element();

        Ok(el.clone())
    }

    fn retain_non_nil_extern_ref(&self, value: Value) -> Result<(), Error> {
        if !value.is_nil() && self.value_type()?.matches(&wasmtime::RefType::EXTERNREF) {
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
    let type_class = root().define_class("TableType", class::object())?;
    type_class.define_method("type", method!(TableType::type_, 0))?;
    type_class.define_method("min_size", method!(TableType::min_size, 0))?;
    type_class.define_method("max_size", method!(TableType::max_size, 0))?;

    let class = root().define_class("Table", class::object())?;
    class.define_singleton_method("new", function!(Table::new, -1))?;

    class.define_method("type", method!(Table::type_, 0))?;
    class.define_method("min_size", method!(Table::min_size, 0))?;
    class.define_method("max_size", method!(Table::max_size, 0))?;

    class.define_method("get", method!(Table::get, 1))?;
    class.define_method("set", method!(Table::set, 2))?;
    class.define_method("grow", method!(Table::grow, 2))?;
    class.define_method("size", method!(Table::size, 0))?;

    Ok(())
}
