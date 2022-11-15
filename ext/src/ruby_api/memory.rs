use super::{memory_type::MemoryType, root, store::Store};
use crate::{error, helpers::WrappedStruct};
use magnus::{
    function, method, r_string::RString, DataTypeFunctions, Error, Module as _, Object, TypedData,
};
use wasmtime::{Extern, Memory as MemoryImpl};

/// @yard
/// Represents a WebAssembly memory.
/// @see https://docs.rs/wasmtime/latest/wasmtime/struct.Memory.html Wasmtime's Rust doc
#[derive(TypedData, Debug)]
#[magnus(class = "Wasmtime::Memory", mark, size, free_immediatly)]
pub struct Memory {
    store: WrappedStruct<Store>,
    inner: MemoryImpl,
}

impl DataTypeFunctions for Memory {
    fn mark(&self) {
        self.store.mark()
    }
}
unsafe impl Send for Memory {}

impl Memory {
    /// @yard
    /// @def new(store, memtype)
    /// @param store [Store]
    /// @param memtype [MemoryType]
    pub fn new(s: WrappedStruct<Store>, memtype: &MemoryType) -> Result<Self, Error> {
        let store = s.get()?;

        let inner = MemoryImpl::new(store.context_mut(), memtype.get().clone())
            .map_err(|e| error!("{}", e))?;

        Ok(Self { store: s, inner })
    }

    pub fn from_inner(store: WrappedStruct<Store>, inner: MemoryImpl) -> Self {
        Self { store, inner }
    }

    /// @yard
    /// Read +size+ bytes starting at +offset+.
    ///
    /// @def read(offset, size)
    /// @param offset [Integer]
    /// @param size [Integer]
    /// @return [String] Binary string of the memory.
    pub fn read(&self, offset: usize, size: usize) -> Result<RString, Error> {
        self.inner
            .data(self.store().context())
            .get(offset..)
            .and_then(|s| s.get(..size))
            .map(RString::from_slice)
            .ok_or_else(|| error!("out of bounds memory access"))
    }

    /// @yard
    /// Write +value+ starting at +offset+.
    ///
    /// @def write(offset, value)
    /// @param offset [Integer]
    /// @param value [String]
    /// @return [void]
    pub fn write(&self, offset: usize, value: RString) -> Result<(), Error> {
        let slice = unsafe { value.as_slice() };

        self.inner
            .write(self.store().context_mut(), offset, slice)
            .map_err(|e| error!("{}", e))
    }

    /// @yard
    /// Grows a memory by +delta+ pages.
    /// Raises if the memory grows beyond its limit.
    ///
    /// @def grow(delta)
    /// @param delta [Integer] The number of pages to grow by.
    /// @return [Integer] The number of pages the memory had before being resized.
    pub fn grow(&self, delta: u64) -> Result<u64, Error> {
        self.inner
            .grow(self.store().context_mut(), delta)
            .map_err(|e| error!("{}", e))
    }

    /// @yard
    /// @return [Integer] The number of pages of the memory.
    pub fn size(&self) -> u64 {
        self.inner.size(self.store().context())
    }

    /// @yard
    /// @return [MemoryType]
    pub fn ty(&self) -> MemoryType {
        self.inner.ty(self.store().context()).into()
    }

    pub fn get(&self) -> MemoryImpl {
        self.inner
    }

    fn store(&self) -> &Store {
        self.store
            .try_convert::<&Store>()
            .expect("Memory.store must be a store")
    }
}

impl From<&Memory> for Extern {
    fn from(memory: &Memory) -> Self {
        Self::Memory(memory.get())
    }
}

pub fn init() -> Result<(), Error> {
    let class = root().define_class("Memory", Default::default())?;
    class.define_singleton_method("new", function!(Memory::new, 2))?;
    class.define_method("read", method!(Memory::read, 2))?;
    class.define_method("write", method!(Memory::write, 2))?;
    class.define_method("grow", method!(Memory::grow, 1))?;
    class.define_method("size", method!(Memory::size, 0))?;
    class.define_method("ty", method!(Memory::ty, 0))?;

    Ok(())
}
