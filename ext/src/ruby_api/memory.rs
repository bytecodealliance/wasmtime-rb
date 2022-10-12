use super::{memory_type::MemoryType, root, store::Store};
use crate::error;
use magnus::{
    function, gc, method, r_string::RString, DataTypeFunctions, Error, Module as _, Object,
    TypedData, Value,
};
use wasmtime::{Extern, Memory as MemoryImpl};

#[derive(TypedData, Debug)]
#[magnus(class = "Wasmtime::Memory", mark, size, free_immediatly)]
pub struct Memory {
    store: Value,
    inner: MemoryImpl,
}

impl DataTypeFunctions for Memory {
    fn mark(&self) {
        gc::mark(&self.store);
    }
}
unsafe impl Send for Memory {}

impl Memory {
    pub fn new(s: Value, memtype: &MemoryType) -> Result<Self, Error> {
        let store: &Store = s.try_convert()?;

        let inner = MemoryImpl::new(store.context_mut(), memtype.get().clone())
            .map_err(|e| error!("{}", e))?;

        Ok(Self { store: s, inner })
    }

    pub fn read(&self, offset: usize, size: usize) -> Result<RString, Error> {
        self.inner
            .data(self.store().context())
            .get(offset..)
            .and_then(|s| s.get(..size))
            .map(RString::from_slice)
            .ok_or_else(|| error!("out of bounds memory access"))
    }

    pub fn write(&self, offset: usize, value: RString) -> Result<(), Error> {
        let slice = unsafe { value.as_slice() };

        self.inner
            .write(self.store().context_mut(), offset, slice)
            .map_err(|e| error!("{}", e))
    }

    pub fn grow(&self, delta: u64) -> Result<u64, Error> {
        self.inner
            .grow(self.store().context_mut(), delta)
            .map_err(|e| error!("{}", e))
    }

    pub fn size(&self) -> u64 {
        self.inner.size(self.store().context())
    }

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
