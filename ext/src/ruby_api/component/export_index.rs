use magnus::{method, prelude::*, Error, Module, RModule, Ruby};
use wasmtime::component::ComponentExportIndex;

/// @yard
/// @rename Wasmtime::Component::ExportIndex
/// Represents a resolved handle to a named export within a {Component}.
/// Can be passed back as the +handle+ argument to {Instance#get_func},
/// {Instance#get_resource}, or {Instance#get_export_index} (either directly,
/// or as an element of an +Array+ handle) to avoid re-resolving the same
/// nested export by name repeatedly.
/// @see https://docs.rs/wasmtime/latest/wasmtime/component/struct.ComponentExportIndex.html Wasmtime's Rust doc
#[magnus::wrap(
    class = "Wasmtime::Component::ExportIndex",
    size,
    free_immediately,
    frozen_shareable
)]
pub struct ExportIndex {
    inner: ComponentExportIndex,
}

unsafe impl Send for ExportIndex {}

impl ExportIndex {
    pub fn from_inner(inner: ComponentExportIndex) -> Self {
        Self { inner }
    }

    pub fn inner(&self) -> ComponentExportIndex {
        self.inner
    }

    /// @yard
    /// @def ==(other)
    /// @param other [Object]
    /// @return [Boolean]
    fn eq(&self, other: &ExportIndex) -> bool {
        self.inner == other.inner
    }
}

pub fn init(ruby: &Ruby, namespace: &RModule) -> Result<(), Error> {
    let class = namespace.define_class("ExportIndex", ruby.class_object())?;
    class.define_method("==", method!(ExportIndex::eq, 1))?;
    class.define_method("eql?", method!(ExportIndex::eq, 1))?;

    Ok(())
}
