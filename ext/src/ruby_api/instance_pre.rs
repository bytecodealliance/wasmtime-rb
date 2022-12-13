use super::{
    instance::Instance,
    root,
    store::{Store, StoreContextValue, StoreData},
};
use crate::{err, helpers::WrappedStruct};
use magnus::{method, DataTypeFunctions, Error, Module as _, TypedData};
use wasmtime::InstancePre as InstancePreImpl;

/// @yard
/// Represents Wasmtime's InstancePre: a pre-validated instance.
/// @see https://docs.rs/wasmtime/latest/wasmtime/struct.InstancePre.html Wasmtime's Rust doc
#[derive(TypedData)]
#[magnus(class = "Wasmtime::InstancePre", mark, free_immediatly)]
pub struct InstancePre {
    has_wasi: bool,
    inner: InstancePreImpl<StoreData>,
    store: WrappedStruct<Store>,
}

impl DataTypeFunctions for InstancePre {
    fn mark(&self) {
        self.store.mark()
    }
}

impl InstancePre {
    pub fn from_inner(
        store: WrappedStruct<Store>,
        inner: InstancePreImpl<StoreData>,
        has_wasi: bool,
    ) -> Self {
        Self {
            store,
            has_wasi,
            inner,
        }
    }

    /// @yard
    /// Creates an {Instance} attached to a {Store}
    /// @def instantiate(store)
    /// @param store [Store]
    /// @return [Instance]
    pub fn instantiate(&self, store: WrappedStruct<Store>) -> Result<Instance, Error> {
        let rs_store = store.get()?;

        if self.has_wasi && !rs_store.context().data().has_wasi_ctx() {
            return err!(
                "Store is missing WASI configuration.\n\n\
                When using `wasi: true`, the Store given to\n\
                `Linker#instantiate` must have a WASI configuration.\n\
                To fix this, provide the `wasi_ctx` when creating the Store:\n\
                    Wasmtime::Store.new(engine, wasi_ctx: WasiCtxBuilder.new)"
            );
        }

        // @TODO instantion error handling
        self.inner
            .instantiate(rs_store.context_mut())
            .map_err(|e| StoreContextValue::from(store).handle_wasm_error(e))
            .map(|instance| Instance::from_inner(store, instance))
    }
}

pub fn init() -> Result<(), Error> {
    let class = root().define_class("InstancePre", Default::default())?;
    class.define_method("instantiate", method!(InstancePre::instantiate, 1))?;
    Ok(())
}
