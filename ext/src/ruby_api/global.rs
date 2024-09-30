use super::{
    convert::{ToRubyValue, ToSym, ToValType, ToWasmVal},
    root,
    store::{Store, StoreContextValue},
};
use crate::error;
use magnus::{
    class, function, gc::Marker, method, prelude::*, typed_data::Obj, DataTypeFunctions, Error,
    Object, Symbol, TypedData, Value,
};
use wasmtime::{Extern, Global as GlobalImpl, Mutability};

#[derive(TypedData)]
#[magnus(
    class = "Wasmtime::GlobalType",
    free_immediately,
    mark,
    unsafe_generics
)]
pub struct GlobalType {
    inner: wasmtime::GlobalType,
}

impl DataTypeFunctions for GlobalType {
    fn mark(&self, _marker: &Marker) {}
}

impl GlobalType {
    pub fn from_inner(inner: wasmtime::GlobalType) -> Self {
        Self { inner }
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
    /// @def type
    /// @return [Symbol] The Wasm type of the global‘s content.
    pub fn type_(&self) -> Result<Symbol, Error> {
        self.inner.content().to_sym()
    }
}

impl From<&GlobalType> for wasmtime::ExternType {
    fn from(global_type: &GlobalType) -> Self {
        Self::Global(global_type.inner.clone())
    }
}

/// @yard
/// @rename Wasmtime::Global
/// Represents a WebAssembly global.
/// @see https://docs.rs/wasmtime/latest/wasmtime/struct.Global.html Wasmtime's Rust doc
#[derive(TypedData)]
#[magnus(class = "Wasmtime::Global", free_immediately, mark, unsafe_generics)]
pub struct Global<'a> {
    store: StoreContextValue<'a>,
    inner: GlobalImpl,
}

impl DataTypeFunctions for Global<'_> {
    fn mark(&self, marker: &Marker) {
        self.store.mark(marker)
    }
}

impl<'a> Global<'a> {
    /// @yard
    /// @def const(store, type, default)
    /// @param store [Store]
    /// @param type [Symbol] The WebAssembly type of the value held by this global.
    /// @param default [Object] The default value of this global.
    /// @return [Global] A constant global.
    pub fn const_(store: Obj<Store>, value_type: Symbol, default: Value) -> Result<Self, Error> {
        Self::new(store, value_type, default, Mutability::Const)
    }

    /// @yard
    /// @def var(store, type, default:)
    /// @param store [Store]
    /// @param type [Symbol] The WebAssembly type of the value held by this global.
    /// @param default [Object] The default value of this global.
    /// @return [Global] A variable global.
    pub fn var(store: Obj<Store>, value_type: Symbol, default: Value) -> Result<Self, Error> {
        Self::new(store, value_type, default, Mutability::Var)
    }

    fn new(
        store: Obj<Store>,
        value_type: Symbol,
        default: Value,
        mutability: Mutability,
    ) -> Result<Self, Error> {
        let wasm_type = value_type.to_val_type()?;
        let wasm_default = default.to_wasm_val(&store.into(), wasm_type.clone())?;
        let inner = GlobalImpl::new(
            store.context_mut(),
            wasmtime::GlobalType::new(wasm_type, mutability),
            wasm_default,
        )
        .map_err(|e| error!("{}", e))?;

        let global = Self {
            store: store.into(),
            inner,
        };

        global.retain_non_nil_extern_ref(default)?;

        Ok(global)
    }

    pub fn from_inner(store: StoreContextValue<'a>, inner: GlobalImpl) -> Self {
        Self { store, inner }
    }

    /// @yard
    /// @def const?
    /// @return [Boolean]
    pub fn is_const(&self) -> Result<bool, Error> {
        self.ty().map(|ty| ty.mutability() == Mutability::Const)
    }

    /// @yard
    /// @def var?
    /// @return [Boolean]
    pub fn is_var(&self) -> Result<bool, Error> {
        self.ty().map(|ty| ty.mutability() == Mutability::Var)
    }

    /// @yard
    /// @def type
    /// @return [Symbol] The Wasm type of the global‘s content.
    pub fn type_(&self) -> Result<Symbol, Error> {
        self.ty()?.content().to_sym()
    }

    /// @yard
    /// @return [Object] The current value of the global.
    pub fn get(&self) -> Result<Value, Error> {
        self.inner
            .get(self.store.context_mut()?)
            .to_ruby_value(&self.store)
    }

    /// @yard
    /// Sets the value of the global. Raises if the global is a +const+.
    /// @def set(value)
    /// @param value [Object] An object that can be converted to the global's type.
    /// @return [nil]
    pub fn set(&self, value: Value) -> Result<(), Error> {
        self.inner
            .set(
                self.store.context_mut()?,
                value.to_wasm_val(&self.store, self.value_type()?)?,
            )
            .map_err(|e| error!("{}", e))
            .and_then(|result| {
                self.retain_non_nil_extern_ref(value)?;
                Ok(result)
            })
    }

    fn ty(&self) -> Result<wasmtime::GlobalType, Error> {
        Ok(self.inner.ty(self.store.context()?))
    }

    fn value_type(&self) -> Result<wasmtime::ValType, Error> {
        self.ty().map(|ty| ty.content().clone())
    }

    fn retain_non_nil_extern_ref(&self, value: Value) -> Result<(), Error> {
        if !value.is_nil() && self.value_type()?.matches(&wasmtime::ValType::EXTERNREF) {
            self.store.retain(value)?;
        }
        Ok(())
    }

    pub fn inner(&self) -> GlobalImpl {
        self.inner
    }
}

impl From<&Global<'_>> for Extern {
    fn from(global: &Global) -> Self {
        Self::Global(global.inner())
    }
}

pub fn init() -> Result<(), Error> {
    let type_class = root().define_class("GlobalType", class::object())?;
    type_class.define_method("const?", method!(GlobalType::is_const, 0))?;
    type_class.define_method("var?", method!(GlobalType::is_var, 0))?;
    type_class.define_method("type", method!(GlobalType::type_, 0))?;

    let class = root().define_class("Global", class::object())?;
    class.define_singleton_method("var", function!(Global::var, 3))?;
    class.define_singleton_method("const", function!(Global::const_, 3))?;

    class.define_method("const?", method!(Global::is_const, 0))?;
    class.define_method("var?", method!(Global::is_var, 0))?;
    class.define_method("type", method!(Global::type_, 0))?;

    class.define_method("get", method!(Global::get, 0))?;
    class.define_method("set", method!(Global::set, 1))?;

    Ok(())
}
