use lazy_static::lazy_static;
use std::cell::RefCell;

use magnus::{
    block::{block_given, block_proc},
    class, function, method,
    rb_sys::AsRawValue,
    typed_data::Obj,
    value::ReprValue,
    Error, Module, Object as _, Value,
};
use rb_sys::{ruby_special_consts::RUBY_Qtrue, VALUE};
use wasmtime::{MpkEnabled, PoolingAllocationConfig as PoolingAllocationConfigImpl};

use crate::{define_rb_intern, err, helpers::SymbolEnum, root};

/// @yard
/// Configuration options used with an engines `allocation_strategy` to change
/// the behavior of the pooling instance allocator.
/// @see https://docs.rs/wasmtime/latest/wasmtime/struct.PoolingAllocationConfig.html Wasmtime's Rust doc
#[derive(Clone)]
#[magnus::wrap(class = "Wasmtime::PoolingAllocationConfig", size, free_immediately)]
pub struct PoolingAllocationConfig {
    inner: RefCell<PoolingAllocationConfigImpl>,
}

impl PoolingAllocationConfig {
    /// @yard
    /// @def new
    /// @return [Wasmtime::PoolingAllocationConfig]
    pub fn new() -> Result<Obj<Self>, Error> {
        let obj = Obj::wrap(Self::from(PoolingAllocationConfigImpl::default()));

        if block_given() {
            let _: Value = block_proc()?.call((obj,))?;
        }

        Ok(obj)
    }

    /// @yard
    /// @def total_memories=
    /// @param total_memories [Integer]
    /// @return [Wasmtime::PoolingAllocationConfig]
    pub fn set_total_memories(rb_self: Obj<Self>, total_memories: u32) -> Result<Obj<Self>, Error> {
        rb_self.borrow_mut()?.total_memories(total_memories);

        Ok(rb_self)
    }

    /// @yard
    /// @def total_tables=
    /// @param total_tables [Integer]
    /// @return [Wasmtime::PoolingAllocationConfig]
    pub fn set_total_tables(rb_self: Obj<Self>, total_tables: u32) -> Result<Obj<Self>, Error> {
        rb_self.borrow_mut()?.total_tables(total_tables);
        Ok(rb_self)
    }

    /// @yard
    /// @def max_memories_per_module=
    /// @param max_memories [Integer]
    /// @return [Wasmtime::PoolingAllocationConfig]
    pub fn set_max_memories_per_module(
        rb_self: Obj<Self>,
        max_memories: u32,
    ) -> Result<Obj<Self>, Error> {
        rb_self.borrow_mut()?.max_memories_per_module(max_memories);
        Ok(rb_self)
    }

    /// @yard
    /// @def max_tables_per_module=
    /// @param max_tables [Integer]
    /// @return [Wasmtime::PoolingAllocationConfig]
    pub fn set_max_tables_per_component(
        rb_self: Obj<Self>,
        max_tables: u32,
    ) -> Result<Obj<Self>, Error> {
        rb_self.borrow_mut()?.max_tables_per_component(max_tables);
        Ok(rb_self)
    }

    /// @yard
    /// @def are_memory_protection_keys_available
    /// @return [Boolean]
    pub fn are_memory_protection_keys_available() -> Result<bool, Error> {
        Ok(wasmtime::PoolingAllocationConfig::are_memory_protection_keys_available())
    }

    /// @yard
    /// @def async_stack_keep_resident=
    /// @param amount [Integer]
    /// @return [Wasmtime::PoolingAllocationConfig]
    pub fn set_async_stack_keep_resident(
        rb_self: Obj<Self>,
        amt: usize,
    ) -> Result<Obj<Self>, Error> {
        rb_self.borrow_mut()?.async_stack_keep_resident(amt);
        Ok(rb_self)
    }

    /// @def async_stack_zeroing=
    /// @param enable [Boolean]
    /// @return [Wasmtime::PoolingAllocationConfig]
    pub fn set_async_stack_zeroing(rb_self: Obj<Self>, enable: Value) -> Result<Obj<Self>, Error> {
        rb_self
            .borrow_mut()?
            .async_stack_zeroing(enable.as_raw() == RUBY_Qtrue as VALUE);
        Ok(rb_self)
    }

    /// @def linear_memory_keep_resident=
    /// @param size [Integer]
    /// @return [Wasmtime::PoolingAllocationConfig]
    pub fn set_linear_memory_keep_resident(
        rb_self: Obj<Self>,
        size: usize,
    ) -> Result<Obj<Self>, Error> {
        rb_self.borrow_mut()?.linear_memory_keep_resident(size);
        Ok(rb_self)
    }

    /// @def max_component_instance_size=
    /// @param size [Integer]
    /// @return [Wasmtime::PoolingAllocationConfig]
    pub fn set_max_component_instance_size(
        rb_self: Obj<Self>,
        size: usize,
    ) -> Result<Obj<Self>, Error> {
        rb_self.borrow_mut()?.max_component_instance_size(size);
        Ok(rb_self)
    }

    /// @def max_core_instance_size=
    /// @param size [Integer]
    /// @return [Wasmtime::PoolingAllocationConfig]
    pub fn set_max_core_instance_size(rb_self: Obj<Self>, size: usize) -> Result<Obj<Self>, Error> {
        rb_self.borrow_mut()?.max_core_instance_size(size);
        Ok(rb_self)
    }

    /// @def max_memories_per_component=
    /// @param max_memories [Integer]
    /// @return [Wasmtime::PoolingAllocationConfig]
    pub fn set_max_memories_per_component(
        rb_self: Obj<Self>,
        max_memories: u32,
    ) -> Result<Obj<Self>, Error> {
        rb_self
            .borrow_mut()?
            .max_memories_per_component(max_memories);
        Ok(rb_self)
    }

    /// @def max_memory_protection_keys=
    /// @param max_keys [Integer]
    /// @return [Wasmtime::PoolingAllocationConfig]
    pub fn set_max_memory_protection_keys(
        rb_self: Obj<Self>,
        max_keys: usize,
    ) -> Result<Obj<Self>, Error> {
        rb_self.borrow_mut()?.max_memory_protection_keys(max_keys);
        Ok(rb_self)
    }

    /// @def max_unused_warm_slots=
    /// @param max_slots [Integer]
    /// @return [Wasmtime::PoolingAllocationConfig]
    pub fn set_max_unused_warm_slots(
        rb_self: Obj<Self>,
        max_slots: u32,
    ) -> Result<Obj<Self>, Error> {
        rb_self.borrow_mut()?.max_unused_warm_slots(max_slots);
        Ok(rb_self)
    }

    /// @def memory_pages=
    /// @param pages [Integer]
    /// @return [Wasmtime::PoolingAllocationConfig]
    pub fn set_memory_pages(rb_self: Obj<Self>, pages: u64) -> Result<Obj<Self>, Error> {
        rb_self.borrow_mut()?.memory_pages(pages);
        Ok(rb_self)
    }

    /// @def memory_protection_keys=
    /// @param keys [Integer]
    /// @return [Wasmtime::PoolingAllocationConfig]
    pub fn set_memory_protection_keys(
        rb_self: Obj<Self>,
        strategy: Value,
    ) -> Result<Obj<Self>, Error> {
        let val = MPK_MAPPING.get(strategy)?;
        rb_self.borrow_mut()?.memory_protection_keys(val);
        Ok(rb_self)
    }

    /// @def table_elements=
    /// @param elements [Integer]
    /// @return [Wasmtime::PoolingAllocationConfig]
    pub fn set_table_elements(rb_self: Obj<Self>, elements: u32) -> Result<Obj<Self>, Error> {
        rb_self.borrow_mut()?.table_elements(elements);
        Ok(rb_self)
    }

    /// @def table_keep_resident=
    /// @param size [Integer]
    /// @return [Wasmtime::PoolingAllocationConfig]
    pub fn set_table_keep_resident(rb_self: Obj<Self>, size: usize) -> Result<Obj<Self>, Error> {
        rb_self.borrow_mut()?.table_keep_resident(size);
        Ok(rb_self)
    }

    /// @def total_component_instances=
    /// @param instances [Integer]
    /// @return [Wasmtime::PoolingAllocationConfig]
    pub fn set_total_component_instances(
        rb_self: Obj<Self>,
        instances: u32,
    ) -> Result<Obj<Self>, Error> {
        rb_self.borrow_mut()?.total_component_instances(instances);
        Ok(rb_self)
    }

    /// @def total_core_instances=
    /// @param instances [Integer]
    /// @return [Wasmtime::PoolingAllocationConfig]
    pub fn set_total_core_instances(
        rb_self: Obj<Self>,
        instances: u32,
    ) -> Result<Obj<Self>, Error> {
        rb_self.borrow_mut()?.total_core_instances(instances);
        Ok(rb_self)
    }

    /// @def total_stacks=
    /// @param stacks [Integer]
    /// @return [Wasmtime::PoolingAllocationConfig]
    pub fn set_total_stacks(rb_self: Obj<Self>, stacks: u32) -> Result<Obj<Self>, Error> {
        rb_self.borrow_mut()?.total_stacks(stacks);
        Ok(rb_self)
    }

    pub fn inspect(rb_self: Obj<Self>) -> Result<String, Error> {
        let inner = format!("{:?}", rb_self.borrow_mut()?);

        Ok(format!(
            "#<Wasmtime::PoolingAllocationConfig inner={}>",
            inner
        ))
    }

    fn borrow_mut(&self) -> Result<std::cell::RefMut<PoolingAllocationConfigImpl>, Error> {
        if let Ok(inner) = self.inner.try_borrow_mut() {
            Ok(inner)
        } else {
            err!("already mutably borrowed")
        }
    }

    pub fn to_inner(&self) -> Result<PoolingAllocationConfigImpl, Error> {
        if let Ok(inner) = self.inner.try_borrow() {
            Ok(inner.clone())
        } else {
            err!("already borrowed")
        }
    }
}

impl From<PoolingAllocationConfigImpl> for PoolingAllocationConfig {
    fn from(inner: PoolingAllocationConfigImpl) -> Self {
        Self {
            inner: RefCell::new(inner),
        }
    }
}

define_rb_intern!(
    AUTO => "auto",
    ENABLE => "enable",
    DISABLE => "disable",
);

lazy_static! {
    pub static ref MPK_MAPPING: SymbolEnum<'static, MpkEnabled> = {
        let mapping = vec![
            (*AUTO, MpkEnabled::Auto),
            (*ENABLE, MpkEnabled::Enable),
            (*DISABLE, MpkEnabled::Disable),
        ];

        SymbolEnum::new("Memory protection keys strategy", mapping)
    };
}

pub fn init() -> Result<(), Error> {
    let class = root().define_class("PoolingAllocationConfig", class::object())?;

    class.define_singleton_method("new", function!(PoolingAllocationConfig::new, 0))?;
    class.define_singleton_method(
        "memory_protection_keys_available?",
        function!(
            PoolingAllocationConfig::are_memory_protection_keys_available,
            0
        ),
    )?;
    class.define_method(
        "total_memories=",
        method!(PoolingAllocationConfig::set_total_memories, 1),
    )?;
    class.define_method(
        "total_tables=",
        method!(PoolingAllocationConfig::set_total_tables, 1),
    )?;
    class.define_method(
        "max_memories_per_module=",
        method!(PoolingAllocationConfig::set_max_memories_per_module, 1),
    )?;
    class.define_method(
        "max_tables_per_module=",
        method!(PoolingAllocationConfig::set_max_tables_per_component, 1),
    )?;
    class.define_method(
        "async_stack_keep_resident=",
        method!(PoolingAllocationConfig::set_async_stack_keep_resident, 1),
    )?;
    class.define_method(
        "async_stack_zeroing=",
        method!(PoolingAllocationConfig::set_async_stack_zeroing, 1),
    )?;
    class.define_method(
        "linear_memory_keep_resident=",
        method!(PoolingAllocationConfig::set_linear_memory_keep_resident, 1),
    )?;
    class.define_method(
        "max_component_instance_size=",
        method!(PoolingAllocationConfig::set_max_component_instance_size, 1),
    )?;
    class.define_method(
        "max_core_instance_size=",
        method!(PoolingAllocationConfig::set_max_core_instance_size, 1),
    )?;
    class.define_method(
        "max_memories_per_component=",
        method!(PoolingAllocationConfig::set_max_memories_per_component, 1),
    )?;
    class.define_method(
        "max_memory_protection_keys=",
        method!(PoolingAllocationConfig::set_max_memory_protection_keys, 1),
    )?;
    class.define_method(
        "max_tables_per_component=",
        method!(PoolingAllocationConfig::set_max_tables_per_component, 1),
    )?;
    class.define_method(
        "max_unused_warm_slots=",
        method!(PoolingAllocationConfig::set_max_unused_warm_slots, 1),
    )?;
    class.define_method(
        "memory_pages=",
        method!(PoolingAllocationConfig::set_memory_pages, 1),
    )?;
    class.define_method(
        "memory_protection_keys=",
        method!(PoolingAllocationConfig::set_memory_protection_keys, 1),
    )?;
    class.define_method(
        "table_elements=",
        method!(PoolingAllocationConfig::set_table_elements, 1),
    )?;
    class.define_method(
        "table_keep_resident=",
        method!(PoolingAllocationConfig::set_table_keep_resident, 1),
    )?;
    class.define_method(
        "total_component_instances=",
        method!(PoolingAllocationConfig::set_total_component_instances, 1),
    )?;
    class.define_method(
        "total_core_instances=",
        method!(PoolingAllocationConfig::set_total_core_instances, 1),
    )?;
    class.define_method(
        "total_stacks=",
        method!(PoolingAllocationConfig::set_total_stacks, 1),
    )?;

    class.define_method("inspect", method!(PoolingAllocationConfig::inspect, 0))?;

    Ok(())
}
