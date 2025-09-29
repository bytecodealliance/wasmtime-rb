use crate::{define_rb_intern, error, root, Memory};
use magnus::{
    class,
    gc::Marker,
    method,
    rb_sys::{AsRawId, AsRawValue, FromRawValue},
    typed_data::Obj,
    value::{IntoId, Lazy, Opaque},
    Class, DataTypeFunctions, Error, Module as _, Ruby, TryConvert, TypedData, Value,
};
#[cfg(ruby_gte_3_0)]
use magnus::{class::object, require, RClass, RModule};
use rb_sys::{rb_ivar_set, rb_obj_freeze, rb_str_new_static};
#[cfg(ruby_gte_3_0)]
use rb_sys::{
    rb_memory_view_entry_t, rb_memory_view_init_as_byte_array, rb_memory_view_register,
    rb_memory_view_t, VALUE,
};
use std::ops::Range;

/// @yard
/// @rename Wasmtime::Memory::UnsafeSlice
/// Represents a slice of a WebAssembly memory. This is useful for creating Ruby
/// strings from Wasm memory without any extra memory allocations.
///
/// The returned {UnsafeSlice} lazily reads the underlying memory, meaning that
/// the actual pointer to the string buffer is not materialzed until
/// {UnsafeSlice#to_str} is called.
#[derive(TypedData)]
#[magnus(
    class = "Wasmtime::Memory::UnsafeSlice",
    free_immediately,
    mark,
    unsafe_generics
)]
pub struct UnsafeSlice<'a> {
    memory: MemoryGuard<'a>,
    range: Range<usize>,
}

define_rb_intern!(IVAR_NAME => "__slice__",);

impl DataTypeFunctions for UnsafeSlice<'_> {
    fn mark(&self, marker: &Marker) {
        self.memory.mark(marker)
    }
}

#[cfg(ruby_gte_3_0)]
fn fiddle_memory_view_class() -> Option<RClass> {
    let fiddle = object().const_get::<_, RModule>("Fiddle").ok()?;
    fiddle.const_get("MemoryView").ok()
}

impl<'a> UnsafeSlice<'a> {
    pub fn new(memory: Obj<Memory<'a>>, range: Range<usize>) -> Result<Self, Error> {
        let memory = MemoryGuard::new(memory)?;
        let slice = Self { memory, range };
        let _ = slice.get_raw_slice()?; // Check that the slice is valid.
        Ok(slice)
    }

    /// @yard
    /// Get this slice as a Fiddle memory view, which can be cheaply read by
    /// other Ruby extensions.
    ///
    /// @def to_memory_view
    /// @return [Fiddle::MemoryView] Memory view of the slice.
    #[cfg(ruby_gte_3_0)]
    pub fn to_memory_view(rb_self: Obj<Self>) -> Result<Value, Error> {
        static CLASS: Lazy<RClass> = Lazy::new(|_| fiddle_memory_view_class().unwrap());
        let ruby = Ruby::get().unwrap();
        ruby.get_inner(&CLASS).new_instance((rb_self,))
    }

    /// @yard
    /// Gets the memory slice as a Ruby string without copying the underlying buffer.
    ///
    /// @def to_str
    /// @return [String] Binary +String+ of the memory.
    pub fn to_str(rb_self: Obj<Self>) -> Result<Value, Error> {
        let raw_slice = rb_self.get_raw_slice()?;
        let id = IVAR_NAME.into_id();
        let rstring = unsafe {
            let val = rb_str_new_static(raw_slice.as_ptr() as _, raw_slice.len() as _);
            rb_ivar_set(val, id.as_raw(), rb_self.as_raw());
            rb_obj_freeze(val)
        };

        Ok(unsafe { Value::from_raw(rstring) })
    }

    fn get_raw_slice(&self) -> Result<&[u8], Error> {
        let mem = self.memory.get()?;

        mem.data()?
            .get(self.range.clone())
            .ok_or_else(|| error!("out of bounds memory access"))
    }

    #[cfg(ruby_gte_3_0)]
    fn register_memory_view(ruby: &Ruby) -> Result<(), Error> {
        let class = Self::class(ruby);

        static ENTRY: rb_memory_view_entry_t = rb_memory_view_entry_t {
            get_func: Some(UnsafeSlice::initialize_memory_view),
            release_func: None,
            available_p_func: Some(UnsafeSlice::is_memory_view_available),
        };

        if unsafe { rb_memory_view_register(class.as_raw(), &ENTRY) } {
            Ok(())
        } else {
            Err(error!("failed to register memory view"))
        }
    }

    #[cfg(ruby_gte_3_0)]
    extern "C" fn initialize_memory_view(
        value: VALUE,
        view: *mut rb_memory_view_t,
        _flags: i32,
    ) -> bool {
        let obj = unsafe { Value::from_raw(value) };
        let Ok(memory) = <Obj<UnsafeSlice>>::try_convert(obj) else {
            return false;
        };
        let Ok(raw_slice) = memory.get_raw_slice() else {
            return false;
        };
        let (ptr, size) = (raw_slice.as_ptr(), raw_slice.len());

        unsafe { rb_memory_view_init_as_byte_array(view, value, ptr as _, size as _, true) }
    }

    #[cfg(ruby_gte_3_0)]
    extern "C" fn is_memory_view_available(value: VALUE) -> bool {
        let obj = unsafe { Value::from_raw(value) };
        let Ok(memory) = <Obj<UnsafeSlice>>::try_convert(obj) else {
            return false;
        };

        memory.get_raw_slice().is_ok()
    }
}

/// A guard that ensures that a memory slice is not invalidated by resizing
pub struct MemoryGuard<'a> {
    memory: Opaque<Obj<Memory<'a>>>,
    original_size: u64,
}

impl<'a> MemoryGuard<'a> {
    pub fn new(memory: Obj<Memory<'a>>) -> Result<Self, Error> {
        let original_size = memory.size()?;

        Ok(Self {
            memory: memory.into(),
            original_size,
        })
    }

    pub fn get(&self) -> Result<&Memory<'a>, Error> {
        let ruby = Ruby::get().unwrap();
        let mem = ruby.get_inner_ref(&self.memory);

        if mem.size()? != self.original_size {
            Err(error!("memory slice was invalidated by resize"))
        } else {
            Ok(mem)
        }
    }

    pub fn mark(&self, marker: &Marker) {
        marker.mark(self.memory)
    }
}

pub fn init(ruby: &Ruby) -> Result<(), Error> {
    let parent = root().define_class("Memory", class::object())?;

    let class = parent.define_class("UnsafeSlice", class::object())?;
    class.define_method("to_str", method!(UnsafeSlice::to_str, 0))?;

    #[cfg(ruby_gte_3_0)]
    if require("fiddle").is_ok() && fiddle_memory_view_class().is_some() {
        UnsafeSlice::register_memory_view(ruby)?;
        class.define_method("to_memory_view", method!(UnsafeSlice::to_memory_view, 0))?;
    }

    Ok(())
}
