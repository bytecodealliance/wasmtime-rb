use super::{engine::Engine, root};
use crate::error;
use magnus::{function, method, Error, Module as _, Object, RString};
use wasmtime::Module as ModuleImpl;

#[derive(Clone)]
#[magnus::wrap(class = "Wasmtime::Module")]
pub struct Module {
    inner: ModuleImpl,
}

impl Module {
    pub fn new(engine: &Engine, wat_or_wasm: RString) -> Result<Self, Error> {
        let eng = engine.get();
        // SAFETY: this string is immediately copied and never moved off the stack
        let module = ModuleImpl::new(eng, unsafe { wat_or_wasm.as_slice() })
            .map_err(|e| error!("Could not build module: {:?}", e.to_string()))?;

        Ok(Self { inner: module })
    }

    pub fn deserialize(engine: &Engine, wat_or_wasm: RString) -> Result<Self, Error> {
        unsafe { ModuleImpl::deserialize(engine.get(), wat_or_wasm.as_slice()) }
            .map(|module| Self { inner: module })
            .map_err(|e| error!("Could not deserialize module: {:?}", e.to_string()))
    }

    pub fn serialize(&self) -> Result<RString, Error> {
        self.get()
            .serialize()
            .map(|bytes| RString::from_slice(&bytes))
            .map_err(|e| error!("{:?}", e))
    }

    pub fn get(&self) -> &ModuleImpl {
        &self.inner
    }
}

pub fn init() -> Result<(), Error> {
    let class = root().define_class("Module", Default::default())?;

    class.define_singleton_method("new", function!(Module::new, 2))?;
    class.define_singleton_method("deserialize", function!(Module::deserialize, 2))?;
    class.define_method("serialize", method!(Module::serialize, 0))?;

    Ok(())
}
