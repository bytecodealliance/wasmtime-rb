use super::{
    convert::{ToRubyValue, ToWasmVal},
    global_type::GlobalType,
    root,
    store::{Store, StoreContextValue},
};
use crate::{error, helpers::WrappedStruct};
use magnus::{
    function, memoize, method, r_typed_data::DataTypeBuilder, DataTypeFunctions, Error,
    Module as _, Object, RClass, TypedData, Value,
};
use wasmtime::{Extern, Global as GlobalImpl};

/// @yard
/// @rename Wasmtime::Global
/// Represents a WebAssembly global.
/// @see https://docs.rs/wasmtime/latest/wasmtime/struct.Global.html Wasmtime's Rust doc
#[derive(Debug)]
pub struct Global<'a> {
    store: StoreContextValue<'a>,
    inner: GlobalImpl,
}

unsafe impl TypedData for Global<'_> {
    fn class() -> magnus::RClass {
        *memoize!(RClass: root().define_class("Global", Default::default()).unwrap())
    }

    fn data_type() -> &'static magnus::DataType {
        memoize!(magnus::DataType: {
            let mut builder = DataTypeBuilder::<Global<'_>>::new("Wasmtime::Global");
            builder.free_immediately();
            builder.mark();
            builder.build()
        })
    }
}

impl DataTypeFunctions for Global<'_> {
    fn mark(&self) {
        self.store.mark()
    }
}

impl<'a> Global<'a> {
    /// @yard
    /// @def new(store, globaltype, value)
    /// @param store [Store]
    /// @param globaltype [GlobalType]
    /// @param value [Object] The value of the global.
    pub fn new(
        s: WrappedStruct<Store>,
        globaltype: &GlobalType,
        value: Value,
    ) -> Result<Self, Error> {
        let store = s.get()?;

        let inner = GlobalImpl::new(
            store.context_mut(),
            globaltype.get().clone(),
            value.to_wasm_val(globaltype.get().content())?,
        )
        .map_err(|e| error!("{}", e))?;

        let global = Self {
            store: s.into(),
            inner,
        };

        global.retain_non_nil_extern_ref(value)?;

        Ok(global)
    }

    pub fn from_inner(store: StoreContextValue<'a>, inner: GlobalImpl) -> Self {
        Self { store, inner }
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
                value.to_wasm_val(&self.value_type()?)?,
            )
            .map_err(|e| error!("{}", e))
            .and_then(|result| {
                self.retain_non_nil_extern_ref(value)?;
                Ok(result)
            })
    }

    /// @yard
    /// @return [GlobalType]
    pub fn ty(&self) -> Result<GlobalType, Error> {
        Ok(self.inner.ty(self.store.context()?).into())
    }

    fn value_type(&self) -> Result<wasmtime::ValType, Error> {
        Ok(self.inner.ty(self.store.context()?).content().clone())
    }

    fn retain_non_nil_extern_ref(&self, value: Value) -> Result<(), Error> {
        if wasmtime::ValType::ExternRef == self.value_type()? && !value.is_nil() {
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
    let class = root().define_class("Global", Default::default())?;
    class.define_singleton_method("new", function!(Global::new, 3))?;
    class.define_method("get", method!(Global::get, 0))?;
    class.define_method("set", method!(Global::set, 1))?;
    class.define_method("ty", method!(Global::ty, 0))?;

    Ok(())
}
