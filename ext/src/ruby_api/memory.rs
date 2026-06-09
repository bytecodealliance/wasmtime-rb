mod unsafe_slice;

use self::unsafe_slice::UnsafeSlice;
use super::{
    root,
    store::{Store, StoreContextValue},
};
use crate::{define_rb_intern, error};
use magnus::{
    class, function, gc::Marker, method, r_string::RString, scan_args, typed_data::Obj,
    DataTypeFunctions, Error, Module as _, Object, Ruby, TypedData, Value,
};

use rb_sys::tracking_allocator::ManuallyTracked;
use wasmtime::{Extern, Memory as MemoryImpl};

const WASM_PAGE_SIZE: u32 = wasmtime_environ::Memory::DEFAULT_PAGE_SIZE;

define_rb_intern!(
    MIN_SIZE => "min_size",
    MAX_SIZE => "max_size",
);

#[derive(TypedData)]
#[magnus(
    class = "Wasmtime::MemoryType",
    free_immediately,
    mark,
    unsafe_generics
)]
pub struct MemoryType {
    inner: wasmtime::MemoryType,
}

impl DataTypeFunctions for MemoryType {
    fn mark(&self, _marker: &Marker) {}
}

impl MemoryType {
    pub fn from_inner(inner: wasmtime::MemoryType) -> Self {
        Self { inner }
    }

    /// @yard
    /// @return [Integer] The minimum number of memory pages.
    pub fn min_size(&self) -> u64 {
        self.inner.minimum()
    }

    /// @yard
    /// @return [Integer, nil] The maximum number of memory pages.
    pub fn max_size(&self) -> Option<u64> {
        self.inner.maximum()
    }
}

impl From<&MemoryType> for wasmtime::ExternType {
    fn from(memory_type: &MemoryType) -> Self {
        Self::Memory(memory_type.inner.clone())
    }
}

/// @yard
/// @rename Wasmtime::Memory
/// Represents a WebAssembly memory.
/// @see https://docs.rs/wasmtime/latest/wasmtime/struct.Memory.html Wasmtime's Rust doc
#[derive(TypedData)]
#[magnus(class = "Wasmtime::Memory", free_immediately, mark, unsafe_generics)]
pub struct Memory<'a> {
    store: StoreContextValue<'a>,
    inner: ManuallyTracked<MemoryImpl>,
}

impl DataTypeFunctions for Memory<'_> {
    fn mark(&self, marker: &Marker) {
        self.store.mark(marker)
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
        let (store,) = args.required;
        let (min,) = kw.required;
        let (max,) = kw.optional;

        let memtype = wasmtime::MemoryType::new(min, max);

        let inner = MemoryImpl::new(store.context_mut(), memtype).map_err(|e| error!("{}", e))?;
        let memsize = inner.data_size(store.context_mut());

        Ok(Self {
            store: store.into(),
            inner: ManuallyTracked::wrap(inner, memsize),
        })
    }

    pub fn from_inner(store: StoreContextValue<'a>, inner: MemoryImpl) -> Result<Self, Error> {
        let memsize = inner.data_size(store.context()?);

        Ok(Self {
            store,
            inner: ManuallyTracked::wrap(inner, memsize),
        })
    }

    /// @yard
    /// @return [Integer] The minimum number of memory pages.
    pub fn min_size(&self) -> Result<u64, Error> {
        Ok(self
            .get_wasmtime_memory()
            .ty(self.store.context()?)
            .minimum())
    }

    /// @yard
    /// @return [Integer, nil] The maximum number of memory pages.
    pub fn max_size(&self) -> Result<Option<u64>, Error> {
        Ok(self
            .get_wasmtime_memory()
            .ty(self.store.context()?)
            .maximum())
    }

    /// @yard
    /// Read +size+ bytes starting at +offset+. Result is a ASCII-8BIT encoded string.
    ///
    /// @def read(offset, size)
    /// @param offset [Integer]
    /// @param size [Integer]
    /// @return [String] Binary +String+ of the memory.
    pub fn read(
        ruby: &Ruby,
        rb_self: Obj<Self>,
        offset: usize,
        size: usize,
    ) -> Result<RString, Error> {
        rb_self
            .get_wasmtime_memory()
            .data(rb_self.store.context()?)
            .get(offset..)
            .and_then(|s| s.get(..size))
            .map(|s| ruby.str_from_slice(s))
            .ok_or_else(|| error!("out of bounds memory access"))
    }

    /// @yard
    /// Read +size+ bytes starting at +offset+. Result is a UTF-8 encoded string.
    ///
    /// @def read_utf8(offset, size)
    /// @param offset [Integer]
    /// @param size [Integer]
    /// @return [String] UTF-8 +String+ of the memory.
    pub fn read_utf8(
        ruby: &Ruby,
        rb_self: Obj<Self>,
        offset: usize,
        size: usize,
    ) -> Result<RString, Error> {
        rb_self
            .get_wasmtime_memory()
            .data(rb_self.store.context()?)
            .get(offset..)
            .and_then(|s| s.get(..size))
            .ok_or_else(|| error!("out of bounds memory access"))
            .and_then(|s| std::str::from_utf8(s).map_err(|e| error!("{}", e)))
            .map(|s| ruby.str_new(s))
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
        ruby: &Ruby,
        rb_self: Obj<Self>,
        offset: usize,
        size: usize,
    ) -> Result<Obj<UnsafeSlice<'a>>, Error> {
        Ok(ruby.obj_wrap(UnsafeSlice::new(rb_self, offset..(offset + size))?))
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

        self.get_wasmtime_memory()
            .write(self.store.context_mut()?, offset, slice)
            .map_err(|e| error!("{}", e))
    }

    /// @yard
    /// Read a little-endian +i32+ starting at +offset+.
    ///
    /// @def read_i32(offset)
    /// @param offset [Integer]
    /// @return [Integer]
    pub fn read_i32(&self, offset: usize) -> Result<i32, Error> {
        Ok(i32::from_le_bytes(self.read_fixed::<4>(offset)?))
    }

    /// @yard
    /// Read a little-endian +u32+ starting at +offset+.
    ///
    /// @def read_u32(offset)
    /// @param offset [Integer]
    /// @return [Integer]
    pub fn read_u32(&self, offset: usize) -> Result<u32, Error> {
        Ok(u32::from_le_bytes(self.read_fixed::<4>(offset)?))
    }

    /// @yard
    /// Read a little-endian +i64+ starting at +offset+.
    ///
    /// @def read_i64(offset)
    /// @param offset [Integer]
    /// @return [Integer]
    pub fn read_i64(&self, offset: usize) -> Result<i64, Error> {
        Ok(i64::from_le_bytes(self.read_fixed::<8>(offset)?))
    }

    /// @yard
    /// Read a little-endian +u64+ starting at +offset+.
    ///
    /// @def read_u64(offset)
    /// @param offset [Integer]
    /// @return [Integer]
    pub fn read_u64(&self, offset: usize) -> Result<u64, Error> {
        Ok(u64::from_le_bytes(self.read_fixed::<8>(offset)?))
    }

    /// @yard
    /// Read a little-endian +f32+ starting at +offset+.
    ///
    /// @def read_f32(offset)
    /// @param offset [Integer]
    /// @return [Float]
    pub fn read_f32(&self, offset: usize) -> Result<f32, Error> {
        Ok(f32::from_le_bytes(self.read_fixed::<4>(offset)?))
    }

    /// @yard
    /// Read a little-endian +f64+ starting at +offset+.
    ///
    /// @def read_f64(offset)
    /// @param offset [Integer]
    /// @return [Float]
    pub fn read_f64(&self, offset: usize) -> Result<f64, Error> {
        Ok(f64::from_le_bytes(self.read_fixed::<8>(offset)?))
    }

    /// @yard
    /// Write a little-endian +i32+ starting at +offset+.
    ///
    /// @def write_i32(offset, value)
    /// @param offset [Integer]
    /// @param value [Integer]
    /// @return [void]
    pub fn write_i32(&self, offset: usize, value: i32) -> Result<(), Error> {
        self.write_fixed(offset, value.to_le_bytes())
    }

    /// @yard
    /// Write a little-endian +u32+ starting at +offset+.
    ///
    /// @def write_u32(offset, value)
    /// @param offset [Integer]
    /// @param value [Integer]
    /// @return [void]
    pub fn write_u32(&self, offset: usize, value: u32) -> Result<(), Error> {
        self.write_fixed(offset, value.to_le_bytes())
    }

    /// @yard
    /// Write a little-endian +i64+ starting at +offset+.
    ///
    /// @def write_i64(offset, value)
    /// @param offset [Integer]
    /// @param value [Integer]
    /// @return [void]
    pub fn write_i64(&self, offset: usize, value: i64) -> Result<(), Error> {
        self.write_fixed(offset, value.to_le_bytes())
    }

    /// @yard
    /// Write a little-endian +u64+ starting at +offset+.
    ///
    /// @def write_u64(offset, value)
    /// @param offset [Integer]
    /// @param value [Integer]
    /// @return [void]
    pub fn write_u64(&self, offset: usize, value: u64) -> Result<(), Error> {
        self.write_fixed(offset, value.to_le_bytes())
    }

    /// @yard
    /// Write a little-endian +f32+ starting at +offset+.
    ///
    /// @def write_f32(offset, value)
    /// @param offset [Integer]
    /// @param value [Float]
    /// @return [void]
    pub fn write_f32(&self, offset: usize, value: f32) -> Result<(), Error> {
        self.write_fixed(offset, value.to_le_bytes())
    }

    /// @yard
    /// Write a little-endian +f64+ starting at +offset+.
    ///
    /// @def write_f64(offset, value)
    /// @param offset [Integer]
    /// @param value [Float]
    /// @return [void]
    pub fn write_f64(&self, offset: usize, value: f64) -> Result<(), Error> {
        self.write_fixed(offset, value.to_le_bytes())
    }

    /// @yard
    /// Read a NUL-terminated C string starting at +offset+ as an ASCII-8BIT
    /// (binary) +String+.
    ///
    /// @def read_cstring(offset)
    /// @param offset [Integer]
    /// @return [String]
    pub fn read_cstring(ruby: &Ruby, rb_self: Obj<Self>, offset: usize) -> Result<RString, Error> {
        let context = rb_self.store.context()?;
        let data = rb_self.get_wasmtime_memory().data(context);

        let bytes: &[u8] = match data.get(offset..) {
            Some(slice) => {
                let end = slice.iter().position(|&b| b == 0).unwrap_or(slice.len());
                &slice[..end]
            }
            None => &[],
        };

        Ok(ruby.str_from_slice(bytes))
    }

    /// @yard
    /// Write +value+'s bytes followed by a NUL terminator at +offset+.
    ///
    /// @def write_cstring(offset, value)
    /// @param offset [Integer]
    /// @param value [String]
    /// @return [void]
    pub fn write_cstring(&self, offset: usize, value: RString) -> Result<(), Error> {
        let slice = unsafe { value.as_slice() };
        if slice.contains(&0) {
            return Err(Error::new(
                Ruby::get_with(value).exception_arg_error(),
                "string contains null byte",
            ));
        }
        let len = slice.len();
        let mut context = self.store.context_mut()?;
        let dst = self
            .get_wasmtime_memory()
            .data_mut(&mut context)
            .get_mut(offset..)
            .and_then(|s| s.get_mut(..len + 1))
            .ok_or_else(|| error!("out of bounds memory access"))?;

        dst[..len].copy_from_slice(slice);
        dst[len] = 0;
        Ok(())
    }

    /// @yard
    /// Grows a memory by +delta+ pages.
    /// Raises if the memory grows beyond its limit.
    ///
    /// @def grow(delta)
    /// @param delta [Integer] The number of pages to grow by.
    /// @return [Integer] The number of pages the memory had before being resized.
    pub fn grow(&self, delta: usize) -> Result<u64, Error> {
        let ret = self
            .get_wasmtime_memory()
            .grow(self.store.context_mut()?, delta as _)
            .map_err(|e| error!("{}", e));

        self.inner
            .increase_memory_usage(delta * (WASM_PAGE_SIZE as usize));

        ret
    }

    /// @yard
    /// @return [Integer] The number of pages of the memory.
    pub fn size(&self) -> Result<u64, Error> {
        Ok(self.get_wasmtime_memory().size(self.store.context()?))
    }

    /// @yard
    /// @return [Integer] The number of bytes of the memory.
    pub fn data_size(&self) -> Result<usize, Error> {
        Ok(self.get_wasmtime_memory().data_size(self.store.context()?))
    }

    pub fn get_wasmtime_memory(&self) -> &MemoryImpl {
        self.inner.get()
    }

    fn data(&self) -> Result<&[u8], Error> {
        Ok(self.get_wasmtime_memory().data(self.store.context()?))
    }

    fn read_fixed<const N: usize>(&self, offset: usize) -> Result<[u8; N], Error> {
        let context = self.store.context()?;

        self.get_wasmtime_memory()
            .data(context)
            .get(offset..)
            .and_then(|s| s.get(..N))
            .and_then(|s| s.try_into().ok())
            .ok_or_else(|| error!("out of bounds memory access"))
    }

    fn write_fixed<const N: usize>(&self, offset: usize, bytes: [u8; N]) -> Result<(), Error> {
        self.get_wasmtime_memory()
            .write(self.store.context_mut()?, offset, &bytes)
            .map_err(|e| error!("{}", e))
    }
}

impl From<&Memory<'_>> for Extern {
    fn from(memory: &Memory) -> Self {
        Self::Memory(*memory.get_wasmtime_memory())
    }
}

pub fn init(ruby: &Ruby) -> Result<(), Error> {
    let type_class = root().define_class("MemoryType", ruby.class_object())?;
    type_class.define_method("min_size", method!(MemoryType::min_size, 0))?;
    type_class.define_method("max_size", method!(MemoryType::max_size, 0))?;

    let class = root().define_class("Memory", ruby.class_object())?;
    class.define_singleton_method("new", function!(Memory::new, -1))?;
    class.define_method("min_size", method!(Memory::min_size, 0))?;
    class.define_method("max_size", method!(Memory::max_size, 0))?;
    class.define_method("read", method!(Memory::read, 2))?;
    class.define_method("read_utf8", method!(Memory::read_utf8, 2))?;
    class.define_method("write", method!(Memory::write, 2))?;
    class.define_method("read_i32", method!(Memory::read_i32, 1))?;
    class.define_method("read_u32", method!(Memory::read_u32, 1))?;
    class.define_method("read_i64", method!(Memory::read_i64, 1))?;
    class.define_method("read_u64", method!(Memory::read_u64, 1))?;
    class.define_method("read_f32", method!(Memory::read_f32, 1))?;
    class.define_method("read_f64", method!(Memory::read_f64, 1))?;
    class.define_method("write_i32", method!(Memory::write_i32, 2))?;
    class.define_method("write_u32", method!(Memory::write_u32, 2))?;
    class.define_method("write_i64", method!(Memory::write_i64, 2))?;
    class.define_method("write_u64", method!(Memory::write_u64, 2))?;
    class.define_method("write_f32", method!(Memory::write_f32, 2))?;
    class.define_method("write_f64", method!(Memory::write_f64, 2))?;
    class.define_method("grow", method!(Memory::grow, 1))?;
    class.define_method("size", method!(Memory::size, 0))?;
    class.define_method("data_size", method!(Memory::data_size, 0))?;
    class.define_method("read_unsafe_slice", method!(Memory::read_unsafe_slice, 2))?;
    class.define_method("read_cstring", method!(Memory::read_cstring, 1))?;
    class.define_method("write_cstring", method!(Memory::write_cstring, 2))?;

    unsafe_slice::init(ruby)?;

    Ok(())
}
