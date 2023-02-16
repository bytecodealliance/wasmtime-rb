mod unsafe_slice;

use self::unsafe_slice::UnsafeSlice;

use super::{
    root,
    store::{Store, StoreContextValue},
};
use crate::{define_data_class, define_rb_intern, error};
use magnus::{
    function, memoize, method, r_string::RString, scan_args, typed_data::DataTypeBuilder,
    typed_data::Obj, DataTypeFunctions, Error, Module as _, Object, RClass, TypedData, Value,
};

use wasmtime::{Extern, Memory as MemoryImpl};

define_rb_intern!(
    MIN_SIZE => "min_size",
    MAX_SIZE => "max_size",
);

/// @yard
/// @rename Wasmtime::Memory
/// Represents a WebAssembly memory.
/// @see https://docs.rs/wasmtime/latest/wasmtime/struct.Memory.html Wasmtime's Rust doc
#[derive(Debug)]
pub struct Memory<'a> {
    store: StoreContextValue<'a>,
    inner: MemoryImpl,
}

unsafe impl TypedData for Memory<'_> {
    fn class() -> magnus::RClass {
        *memoize!(RClass: define_data_class!(root(), "Memory"))
    }

    fn data_type() -> &'static magnus::DataType {
        memoize!(magnus::DataType: {
            let mut builder = DataTypeBuilder::<Memory<'_>>::new("Wasmtime::Memory");
            builder.free_immediately();
            builder.mark();
            builder.build()
        })
    }
}

impl DataTypeFunctions for Memory<'_> {
    fn mark(&self) {
        self.store.mark()
    }
}
unsafe impl Send for Memory<'_> {}

impl<'a> Memory<'a> {
    /// @yard
    /// @def new(store, min_size:, max_size: nil)
    /// @param store [Store]
    /// @param min_size [Integer] The minimum memory pages.
    /// @param max_size [Integer, nil] The maximum memory pages.
    pub fn new(args: &[Value]) -> Result<Self, Error> {
        let args = scan_args::scan_args::<(Obj<Store>,), (), (), (), _, ()>(args)?;
        let kw = scan_args::get_kwargs::<_, (u32,), (Option<u32>,), ()>(
            args.keywords,
            &[*MIN_SIZE],
            &[*MAX_SIZE],
        )?;
        let (s,) = args.required;
        let (min,) = kw.required;
        let (max,) = kw.optional;
        let store = s.get();

        let memtype = wasmtime::MemoryType::new(min, max);
        let inner = MemoryImpl::new(store.context_mut(), memtype).map_err(|e| error!("{}", e))?;

        Ok(Self {
            store: s.into(),
            inner,
        })
    }

    pub fn from_inner(store: StoreContextValue<'a>, inner: MemoryImpl) -> Self {
        Self { store, inner }
    }

    /// @yard
    /// @return [Integer] The minimum number of memory pages.
    pub fn min_size(&self) -> Result<u64, Error> {
        Ok(self.inner.ty(self.store.context()?).minimum())
    }

    /// @yard
    /// @return [Integer, nil] The maximum number of memory pages.
    pub fn max_size(&self) -> Result<Option<u64>, Error> {
        Ok(self.inner.ty(self.store.context()?).maximum())
    }

    /// @yard
    /// Read +size+ bytes starting at +offset+. Result is a ASCII-8BIT encoded string.
    ///
    /// @def read(offset, size)
    /// @param offset [Integer]
    /// @param size [Integer]
    /// @return [String] Binary +String+ of the memory.
    pub fn read(&self, offset: usize, size: usize) -> Result<RString, Error> {
        self.inner
            .data(self.store.context()?)
            .get(offset..)
            .and_then(|s| s.get(..size))
            .map(RString::from_slice)
            .ok_or_else(|| error!("out of bounds memory access"))
    }

    /// @yard
    /// Read +size+ bytes starting at +offset+. Result is a UTF-8 encoded string.
    ///
    /// @def read_utf8(offset, size)
    /// @param offset [Integer]
    /// @param size [Integer]
    /// @return [String] UTF-8 +String+ of the memory.
    pub fn read_utf8(&self, offset: usize, size: usize) -> Result<RString, Error> {
        self.inner
            .data(self.store.context()?)
            .get(offset..)
            .and_then(|s| s.get(..size))
            .ok_or_else(|| error!("out of bounds memory access"))
            .and_then(|s| std::str::from_utf8(s).map_err(|e| error!("{}", e)))
            .map(RString::new)
    }

    /// @yard
    /// Read +size+ bytes starting at +offset+ into an {UnsafeSlice}. This
    /// provides a way to read a slice of memory without copying the underlying
    /// data.
    ///
    /// The returned {UnsafeSlice} lazily reads the underlying memory, meaning that
    /// the actual pointer to the string buffer is not materialzed until
    /// {UnsafeSlice#to_str} is called.
    ///
    /// SAFETY: Resizing the memory (as with {Wasmtime::Memory#grow}) will
    /// invalidate the {UnsafeSlice}, and future attempts to read the slice will raise
    /// an error.  However, it is not possible to invalidate the Ruby +String+
    /// object after calling {Memory::UnsafeSlice#to_str}. As such, the caller must ensure
    /// that the Wasmtime {Memory} is not resized while holding the Ruby string.
    /// Failing to do so could result in the +String+ buffer pointing to invalid
    /// memory.
    ///
    /// In general, you should prefer using {Memory#read} or {Memory#read_utf8}
    /// over this method unless you know what you're doing.
    ///
    /// @def read_unsafe_slice(offset, size)
    /// @param offset [Integer]
    /// @param size [Integer]
    /// @return [Wasmtime::Memory::UnsafeSlice] Slice of the memory.
    pub fn read_unsafe_slice(
        rb_self: Obj<Self>,
        offset: usize,
        size: usize,
    ) -> Result<Obj<UnsafeSlice<'a>>, Error> {
        Ok(Obj::wrap(UnsafeSlice::new(
            rb_self,
            offset..(offset + size),
        )?))
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
            .write(self.store.context_mut()?, offset, slice)
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
            .grow(self.store.context_mut()?, delta)
            .map_err(|e| error!("{}", e))
    }

    /// @yard
    /// @return [Integer] The number of pages of the memory.
    pub fn size(&self) -> Result<u64, Error> {
        Ok(self.inner.size(self.store.context()?))
    }

    pub fn get(&self) -> MemoryImpl {
        self.inner
    }

    fn data(&self) -> Result<&[u8], Error> {
        Ok(self.inner.data(self.store.context()?))
    }
}

impl From<&Memory<'_>> for Extern {
    fn from(memory: &Memory) -> Self {
        Self::Memory(memory.get())
    }
}

pub fn init() -> Result<(), Error> {
    let class = Memory::class();
    class.define_singleton_method("new", function!(Memory::new, -1))?;
    class.define_method("min_size", method!(Memory::min_size, 0))?;
    class.define_method("max_size", method!(Memory::max_size, 0))?;
    class.define_method("read", method!(Memory::read, 2))?;
    class.define_method("read_utf8", method!(Memory::read_utf8, 2))?;
    class.define_method("write", method!(Memory::write, 2))?;
    class.define_method("grow", method!(Memory::grow, 1))?;
    class.define_method("size", method!(Memory::size, 0))?;
    class.define_method("read_unsafe_slice", method!(Memory::read_unsafe_slice, 2))?;

    unsafe_slice::init()?;

    Ok(())
}
