use rb_sys::tracking_allocator::ManuallyTracked;
use wasmtime::{LinearMemory, MemoryCreator};
use wasmtime_environ::{Memory, MemoryPlan};
use wasmtime_runtime::{DefaultMemoryCreator, RuntimeLinearMemory, RuntimeMemoryCreator};

pub(crate) struct TrackedLinearMemory {
    inner: ManuallyTracked<Box<dyn RuntimeLinearMemory>>,
}

impl TrackedLinearMemory {
    pub(crate) fn new(inner: Box<dyn RuntimeLinearMemory>) -> Self {
        let memsize = inner.byte_size();

        Self {
            inner: ManuallyTracked::wrap(inner, memsize),
        }
    }
}

unsafe impl LinearMemory for TrackedLinearMemory {
    fn byte_size(&self) -> usize {
        self.inner.get().byte_size()
    }

    fn maximum_byte_size(&self) -> Option<usize> {
        self.inner.get().maximum_byte_size()
    }

    fn grow_to(&mut self, size: usize) -> anyhow::Result<()> {
        self.inner.increase_memory_usage(size);
        self.inner.get_mut().grow_to(size)
    }

    fn wasm_accessible(&self) -> std::ops::Range<usize> {
        self.inner.get().wasm_accessible()
    }

    fn as_ptr(&self) -> *mut u8 {
        self.wasm_accessible().start as *mut u8
    }
}

/// Wrapper around the default memory creator that reports `mmap` allocations to
/// the Ruby VM.
///
/// Note: This is needed because the Rust allocator is not used for `mmap`, so
/// we have to manually track the allocations.
pub(crate) struct TrackedMemoryCreator(DefaultMemoryCreator);

impl TrackedMemoryCreator {
    pub(crate) fn new() -> Self {
        Self(DefaultMemoryCreator)
    }
}

unsafe impl MemoryCreator for TrackedMemoryCreator {
    fn new_memory(
        &self,
        ty: wasmtime::MemoryType,
        minimum: usize,
        maximum: Option<usize>,
        _reserved_size_in_bytes: Option<usize>,
        _guard_size_in_bytes: usize,
    ) -> anyhow::Result<Box<dyn wasmtime::LinearMemory>, String> {
        let default_memory_creator = DefaultMemoryCreator {};
        let memory = Memory {
            minimum: ty.minimum(),
            maximum: ty.maximum(),
            shared: ty.is_shared(),
            memory64: ty.is_64(),
        };
        let tunables = wasmtime_environ::Tunables::default_host();
        let plan = MemoryPlan::for_memory(memory, &tunables);
        let base = default_memory_creator
            .new_memory(&plan, minimum, maximum, None)
            .map_err(|e| e.to_string())?;
        let mem = TrackedLinearMemory::new(base);
        let mem = Box::new(mem);

        Ok(mem)
    }
}
