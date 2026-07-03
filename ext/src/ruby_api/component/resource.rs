use std::cell::RefCell;

use crate::error;
use crate::ruby_api::store::Store;
use magnus::{
    gc::Marker, method, prelude::*, typed_data::Obj, DataTypeFunctions, Error, Module, RModule,
    Ruby, TypedData,
};
use wasmtime::component::{ResourceAny as ResourceAnyImpl, ResourceType as ResourceTypeImpl};

/// @yard
/// @rename Wasmtime::Component::ResourceType
/// Represents the type of a WIT +resource+. Two {ResourceType}s are equal iff
/// they describe the same resource type in the same {Component} instantiation.
/// Returned by {Instance#get_resource} and {Resource#type}. There is no public
/// constructor.
/// @see https://docs.rs/wasmtime/latest/wasmtime/component/struct.ResourceType.html Wasmtime's Rust doc
#[magnus::wrap(
    class = "Wasmtime::Component::ResourceType",
    size,
    free_immediately,
    frozen_shareable
)]
pub struct ResourceType {
    inner: ResourceTypeImpl,
}

unsafe impl Send for ResourceType {}

impl ResourceType {
    pub fn from_inner(inner: ResourceTypeImpl) -> Self {
        Self { inner }
    }

    /// @yard
    /// @def ==(other)
    /// @param other [Object]
    /// @return [Boolean]
    fn eq(&self, other: &ResourceType) -> bool {
        self.inner == other.inner
    }
}

/// @yard
/// @rename Wasmtime::Component::Resource
/// Represents a handle to a WIT +resource+ (either +own<T>+ or +borrow<T>+),
/// as produced by calling a Wasm component export that returns a resource, or
/// passed as an argument to one that accepts one.
///
/// IMPORTANT: a {Resource} MUST be explicitly destroyed with {#resource_drop}
/// once it is no longer needed. This holds for +own+ *and* +borrow+ handles
/// alike: both hold state in the owning {Store} that must be released. There
/// is no automatic finalizer: Ruby's GC may run at times where it would be
/// unsafe to re-enter the {Store} (e.g. to invoke a guest-defined
/// destructor), so failing to call {#resource_drop} will leak store-side
/// resource-table state for the life of the {Store}.
///
/// Passing an +own<T>+ {Resource} into a function parameter transfers
/// ownership of the underlying resource to the callee. Using the same
/// {Resource} object afterwards (including calling {#resource_drop} on it)
/// may then raise, since the store-side handle it pointed to no longer
/// belongs to the host.
///
/// A {Resource} is tied to the {Store} it was obtained from; passing one into
/// a function call on a different {Store} raises.
/// @see https://docs.rs/wasmtime/latest/wasmtime/component/struct.ResourceAny.html Wasmtime's Rust doc for +ResourceAny+
#[derive(TypedData)]
#[magnus(class = "Wasmtime::Component::Resource", size, mark, free_immediately)]
pub struct Resource {
    store: Obj<Store>,
    inner: RefCell<Option<ResourceAnyImpl>>,
}

unsafe impl Send for Resource {}

impl DataTypeFunctions for Resource {
    fn mark(&self, marker: &Marker) {
        marker.mark(self.store);
    }
}

impl Resource {
    pub fn from_inner(store: Obj<Store>, inner: ResourceAnyImpl) -> Self {
        Self {
            store,
            inner: RefCell::new(Some(inner)),
        }
    }

    /// Returns the live inner resource handle, without consuming it.
    /// Errors if the resource has already been dropped.
    pub(crate) fn get(&self) -> Result<ResourceAnyImpl, Error> {
        (*self.inner.borrow()).ok_or_else(|| error!("Resource has already been dropped"))
    }

    /// The `Store` this resource was obtained from.
    pub(crate) fn store(&self) -> Obj<Store> {
        self.store
    }

    /// @yard
    /// @def owned?
    /// @return [Boolean] whether this is an +own<T>+ (+true+) or +borrow<T>+
    ///   (+false+) handle.
    fn owned(&self) -> Result<bool, Error> {
        Ok(self.get()?.owned())
    }

    /// @yard
    /// @def type
    /// @return [ResourceType]
    fn type_(&self) -> Result<ResourceType, Error> {
        Ok(ResourceType::from_inner(self.get()?.ty()))
    }

    /// @yard
    /// Explicitly destroys this resource, releasing store-side state and
    /// invoking the guest-defined destructor if applicable. MUST be called
    /// exactly once for every {Resource}, including borrows.
    /// @def resource_drop
    /// @return [nil]
    fn resource_drop(&self) -> Result<(), Error> {
        let resource_any = self
            .inner
            .borrow_mut()
            .take()
            .ok_or_else(|| error!("Resource has already been dropped"))?;

        resource_any
            .resource_drop(self.store.context_mut())
            .map_err(|e| error!("{}", e))
    }
}

pub fn init(ruby: &Ruby, namespace: &RModule) -> Result<(), Error> {
    let resource_type = namespace.define_class("ResourceType", ruby.class_object())?;
    resource_type.define_method("==", method!(ResourceType::eq, 1))?;
    resource_type.define_method("eql?", method!(ResourceType::eq, 1))?;

    let resource = namespace.define_class("Resource", ruby.class_object())?;
    resource.define_method("owned?", method!(Resource::owned, 0))?;
    resource.define_method("type", method!(Resource::type_, 0))?;
    resource.define_method("resource_drop", method!(Resource::resource_drop, 0))?;

    Ok(())
}
