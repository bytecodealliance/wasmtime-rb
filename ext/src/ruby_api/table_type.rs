use super::{
    convert::{ToSym, ToValType},
    root,
};
use magnus::{function, method, scan_args, Error, Module as _, Object, Symbol, Value};
use wasmtime::TableType as TableTypeImpl;

/// @yard
/// @see https://docs.rs/wasmtime/latest/wasmtime/struct.TableType.html Wasmtime's Rust doc
#[derive(Clone, Debug)]
#[magnus::wrap(class = "Wasmtime::TableType")]
pub struct TableType {
    inner: TableTypeImpl,
}

impl TableType {
    /// @yard
    /// @def new(element, min, max = nil)
    /// @param element [Symbol] The type of the elements in the {Table}.
    /// @param min [Integer] The minimum {Table} size.
    /// @param max [Integer, nil] The maximum {Table} size.
    pub fn new(args: &[Value]) -> Result<Self, Error> {
        let args = scan_args::scan_args::<(Value, u32), (Option<u32>,), (), (), (), ()>(args)?;
        let (ty, min) = args.required;
        let (max,) = args.optional;
        let inner = TableTypeImpl::new(ty.to_val_type()?, min, max);
        Ok(Self { inner })
    }

    pub fn get(&self) -> &TableTypeImpl {
        &self.inner
    }

    /// @yard
    /// @return [Symbol] The type of elements in the {Table}.
    pub fn element(&self) -> Symbol {
        self.inner.element().to_sym()
    }

    /// @yard
    /// @return [Integer] The minimum size of the {Table}.
    pub fn minimum(&self) -> u32 {
        self.inner.minimum()
    }

    /// @yard
    /// @return [Integer, nil] The maximum size of the {Table}.
    pub fn maximum(&self) -> Option<u32> {
        self.inner.maximum()
    }
}

pub fn init() -> Result<(), Error> {
    let class = root().define_class("TableType", Default::default())?;

    class.define_singleton_method("new", function!(TableType::new, -1))?;
    class.define_method("element", method!(TableType::element, 0))?;
    class.define_method("minimum", method!(TableType::minimum, 0))?;
    class.define_method("maximum", method!(TableType::maximum, 0))?;
    Ok(())
}

impl From<TableTypeImpl> for TableType {
    fn from(inner: TableTypeImpl) -> Self {
        Self { inner }
    }
}
