use super::root;
use magnus::{function, method, scan_args, Error, Module as _, Object, Value};
use wasmtime::MemoryType as MemoryTypeImpl;

/// @yard
/// @see https://docs.rs/wasmtime/latest/wasmtime/struct.MemoryType.html Wasmtime's Rust doc
#[derive(Clone, Debug)]
#[magnus::wrap(class = "Wasmtime::MemoryType")]
pub struct MemoryType {
    inner: MemoryTypeImpl,
}

impl MemoryType {
    /// @yard
    /// @def new(min, max = nil)
    /// @param min [Integer] The minimum memory pages.
    /// @param max [Integer, nil] The maximum memory pages.
    pub fn new(args: &[Value]) -> Result<Self, Error> {
        let args = scan_args::scan_args::<(u32,), (Option<u32>,), (), (), (), ()>(args)?;
        let (min,) = args.required;
        let (max,) = args.optional;
        let inner = MemoryTypeImpl::new(min, max);
        Ok(Self { inner })
    }

    pub fn get(&self) -> &MemoryTypeImpl {
        &self.inner
    }

    /// @yard
    /// @return [Integer] The minimum memory pages.
    pub fn minimum(&self) -> u64 {
        self.inner.minimum()
    }

    /// @yard
    /// @return [Integer, nil] The maximum memory pages.
    pub fn maximum(&self) -> Option<u64> {
        self.inner.maximum()
    }
}

pub fn init() -> Result<(), Error> {
    let class = root().define_class("MemoryType", Default::default())?;

    class.define_singleton_method("new", function!(MemoryType::new, -1))?;
    class.define_method("minimum", method!(MemoryType::minimum, 0))?;
    class.define_method("maximum", method!(MemoryType::maximum, 0))?;
    Ok(())
}

impl From<MemoryTypeImpl> for MemoryType {
    fn from(inner: MemoryTypeImpl) -> Self {
        Self { inner }
    }
}
