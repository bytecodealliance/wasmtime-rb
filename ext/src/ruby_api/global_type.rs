use super::{
    convert::{ToSym, ToValType},
    root,
};
use magnus::{function, method, Error, Module as _, Object, Symbol};
use wasmtime::{GlobalType as GlobalTypeImpl, Mutability};

/// @yard
/// @see https://docs.rs/wasmtime/latest/wasmtime/struct.GlobalType.html Wasmtime's Rust doc
#[derive(Clone, Debug)]
#[magnus::wrap(class = "Wasmtime::GlobalType")]
pub struct GlobalType {
    inner: GlobalTypeImpl,
}

impl GlobalType {
    /// @yard
    /// @def const(content)
    /// @param content [Symbol] The type of {Global}‘s content.
    /// @return [GlobalType] A constant GlobalType.
    pub fn const_(content: Symbol) -> Result<Self, Error> {
        Self::new(content, Mutability::Const)
    }

    /// @yard
    /// @def var(content)
    /// @param content [Symbol] The type of {Global}‘s content.
    /// @return [GlobalType] A variable GlobalType.
    pub fn var(content: Symbol) -> Result<Self, Error> {
        Self::new(content, Mutability::Var)
    }

    /// @yard
    /// @def const?
    /// @return [Boolean]
    pub fn is_const(&self) -> bool {
        self.inner.mutability() == Mutability::Const
    }

    /// @yard
    /// @def var?
    /// @return [Boolean]
    pub fn is_var(&self) -> bool {
        self.inner.mutability() == Mutability::Var
    }

    /// @yard
    /// @return [Symbol] The Wasm type of the {Global}‘s content.
    pub fn content(&self) -> Symbol {
        self.inner.content().clone().to_sym()
    }

    pub fn get(&self) -> &GlobalTypeImpl {
        &self.inner
    }

    fn new(content: Symbol, mutability: Mutability) -> Result<Self, Error> {
        let inner = GlobalTypeImpl::new(content.to_val_type()?, mutability);

        Ok(Self { inner })
    }
}

pub fn init() -> Result<(), Error> {
    let class = root().define_class("GlobalType", Default::default())?;

    class.define_singleton_method("const", function!(GlobalType::const_, 1))?;
    class.define_singleton_method("var", function!(GlobalType::var, 1))?;
    class.define_method("const?", method!(GlobalType::is_const, 0))?;
    class.define_method("var?", method!(GlobalType::is_var, 0))?;
    class.define_method("content", method!(GlobalType::content, 0))?;
    Ok(())
}

impl From<GlobalTypeImpl> for GlobalType {
    fn from(inner: GlobalTypeImpl) -> Self {
        Self { inner }
    }
}
