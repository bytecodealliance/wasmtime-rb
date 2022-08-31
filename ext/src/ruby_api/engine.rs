use super::{config::Config, root};
use crate::error;
use magnus::{function, method, Error, Module, Object};
use wasmtime::Engine as EngineImpl;

#[derive(Clone)]
#[magnus::wrap(class = "Wasmtime::Engine")]
pub struct Engine {
    inner: EngineImpl,
}

impl Engine {
    pub fn new(config: &Config) -> Result<Self, Error> {
        let config = config.get();
        let inner = EngineImpl::new(&config).map_err(|e| error!("{}", e))?;

        Ok(Self { inner })
    }

    pub fn get(&self) -> &EngineImpl {
        &self.inner
    }

    pub fn is_equal(&self, other: &Engine) -> bool {
        EngineImpl::same(self.get(), other.get())
    }
}

pub fn init() -> Result<(), Error> {
    let class = root().define_class("Engine", Default::default())?;

    class.define_singleton_method("new", function!(Engine::new, 1))?;

    class.define_method("==", method!(Engine::is_equal, 1))?;

    Ok(())
}
