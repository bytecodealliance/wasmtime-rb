use crate::{define_data_class, define_rb_intern, error, helpers::WrappedStruct, Memory};
use magnus::{
    class::object,
    gc, memoize, method,
    r_typed_data::DataTypeBuilder,
    rb_sys::{AsRawId, AsRawValue, FromRawValue},
    require,
    value::Id,
    DataTypeFunctions, Error, Module as _, RClass, RModule, TypedData, Value,
};
use rb_sys::{
    rb_ivar_set, rb_memory_view_entry_t, rb_memory_view_init_as_byte_array,
    rb_memory_view_register, rb_memory_view_t, rb_obj_freeze, rb_str_new_static, VALUE,
};
use std::ops::Range;

/// @yard
/// @rename Wasmtime::Memory::Slice
/// Represents a slice of a WebAssembly memory. This is useful for creating Ruby
/// strings from WASM memory without any extra memory allocations.
#[derive(Debug)]
pub struct Slice<'a> {
    memory: MemoryGuard<'a>,
    range: Range<usize>,
}

define_rb_intern!(IVAR_NAME => "@__slice__",);

unsafe impl TypedData for Slice<'_> {
    fn class() -> magnus::RClass {
        *memoize!(RClass: define_data_class!(Memory::class(), "Slice"))
    }

    fn data_type() -> &'static magnus::DataType {
        memoize!(magnus::DataType: {
            let mut builder = DataTypeBuilder::<Slice>::new("Wasmtime::Memory::Slice");
            builder.free_immediately();
            builder.mark();
            builder.build()
        })
    }
}

impl DataTypeFunctions for Slice<'_> {
    fn mark(&self) {
        self.memory.mark()
    }
}

fn fiddle_memory_view_class() -> Option<RClass> {
    let fiddle = object().const_get::<_, RModule>("Fiddle").ok()?;
    fiddle.const_get("MemoryView").ok()
}

impl<'a> Slice<'a> {
    pub fn new(memory: WrappedStruct<Memory<'a>>, range: Range<usize>) -> Result<Self, Error> {
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
    pub fn to_memory_view(rb_self: WrappedStruct<Self>) -> Result<Value, Error> {
        let klass = *memoize!(RClass: {
            let c = fiddle_memory_view_class().unwrap();
            gc::register_mark_object(c);
            c
        });

        klass.new_instance((rb_self,))
    }

    /// @yard
    /// Gets the memory slice as a Ruby string without copying the underlying buffer.
    ///
    /// @def to_str
    /// @return [String] Binary +String+ of the memory.
    pub fn to_str(rb_self: WrappedStruct<Self>) -> Result<Value, Error> {
        let raw_slice = rb_self.get()?.get_raw_slice()?;
        let id: Id = (*IVAR_NAME).into();
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

    fn register_memory_view() -> Result<(), Error> {
        let class = Self::class();

        static ENTRY: rb_memory_view_entry_t = rb_memory_view_entry_t {
            get_func: Some(Slice::initialize_memory_view),
            release_func: None,
            available_p_func: Some(Slice::is_memory_view_available),
        };

        if unsafe { rb_memory_view_register(class.as_raw(), &ENTRY) } {
            Ok(())
        } else {
            Err(error!("failed to register memory view"))
        }
    }

    extern "C" fn initialize_memory_view(
        value: VALUE,
        view: *mut rb_memory_view_t,
        _flags: i32,
    ) -> bool {
        let obj = unsafe { Value::from_raw(value) };
        let Ok(memory) = obj.try_convert::<WrappedStruct<Slice>>() else { return false };
        let Ok(memory) = memory.get() else { return false; };
        let Ok(raw_slice) = memory.get_raw_slice() else { return false; };
        let (ptr, size) = (raw_slice.as_ptr(), raw_slice.len());

        unsafe { rb_memory_view_init_as_byte_array(view, value, ptr as _, size as _, true) }
    }

    extern "C" fn is_memory_view_available(value: VALUE) -> bool {
        let obj = unsafe { Value::from_raw(value) };
        let Ok(memory) = obj.try_convert::<WrappedStruct<Slice>>() else { return false };
        let Ok(memory) = memory.get() else { return false; };

        memory.get_raw_slice().is_ok()
    }
}

/// A guard that ensures that a memory slice is not invalidated by resizing
#[derive(Debug)]
pub struct MemoryGuard<'a> {
    memory: WrappedStruct<Memory<'a>>,
    original_size: u64,
}

impl<'a> MemoryGuard<'a> {
    pub fn new(memory: WrappedStruct<Memory<'a>>) -> Result<Self, Error> {
        let original_size = memory.get()?.size()?;

        Ok(Self {
            memory,
            original_size,
        })
    }

    pub fn get(&self) -> Result<&Memory<'a>, Error> {
        let mem = self.memory.get()?;

        if mem.size()? != self.original_size {
            Err(error!("memory slice was invalidated by resize"))
        } else {
            Ok(mem)
        }
    }

    pub fn mark(&self) {
        self.memory.mark()
    }
}

pub fn init() -> Result<(), Error> {
    Slice::class().define_method("to_str", method!(Slice::to_str, 0))?;

    if require("fiddle").is_ok() && fiddle_memory_view_class().is_some() {
        Slice::register_memory_view()?;
        Slice::class().define_method("to_memory_view", method!(Slice::to_memory_view, 0))?;
    }

    Ok(())
}
