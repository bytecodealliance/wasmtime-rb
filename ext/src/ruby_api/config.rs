use super::root;
use magnus::{function, method, Error, Module, Object};
use std::cell::RefCell;
use wasmtime::Config as ConfigImpl;

/// @yard
/// Wasmtime {Engine} configuration.
/// @see https://docs.rs/wasmtime/latest/wasmtime/struct.Config.html Wasmtime's Rust doc
#[derive(Clone, Debug)]
#[magnus::wrap(class = "Wasmtime::Config")]
pub struct Config {
    inner: RefCell<ConfigImpl>,
}

impl Config {
    /// @yard
    /// @return [Config]
    pub fn new() -> Self {
        Self {
            inner: RefCell::new(ConfigImpl::new()),
        }
    }

    pub fn get(&self) -> ConfigImpl {
        self.inner.borrow().clone()
    }

    /// @yard
    /// @def epoch_interruption=(enabled)
    /// @param enabled [Boolean]
    pub fn set_epoch_interruption(&self, enabled: bool) {
        self.inner.borrow_mut().epoch_interruption(enabled);
    }

    /// @yard
    /// @def max_wasm_stack=(size)
    /// @param size [Integer]
    pub fn set_max_wasm_stack(&self, size: usize) {
        self.inner.borrow_mut().max_wasm_stack(size);
    }

    /// @yard
    /// @def wasm_multi_memory=(enabled)
    /// @param enabled [Boolean]
    pub fn set_wasm_multi_memory(&self, enabled: bool) {
        self.inner.borrow_mut().wasm_multi_memory(enabled);
    }
}

pub fn init() -> Result<(), Error> {
    let class = root().define_class("Config", Default::default())?;

    class.define_singleton_method("new", function!(Config::new, 0))?;

    class.define_method(
        "epoch_interruption=",
        method!(Config::set_epoch_interruption, 1),
    )?;

    class.define_method("max_wasm_stack=", method!(Config::set_max_wasm_stack, 1))?;

    class.define_method(
        "wasm_multi_memory=",
        method!(Config::set_wasm_multi_memory, 1),
    )?;

    Ok(())
}
