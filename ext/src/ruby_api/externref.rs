//! Ruby values held by Wasmtime externrefs.

// The store uses a map of integer handles to Ruby values. The integer handles are what is sent
// and returned from Wasmtime. Handles are monotically increased and never reused. This allows
// the Ruby values to be moved by the garbage collector.
use crate::error;
use magnus::{
    gc::{Compactor, Marker},
    value::Opaque,
    Error, Ruby, Value,
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, MutexGuard, Weak},
};

#[derive(Default)]
struct Entries {
    next_handle: u64,
    values: HashMap<u64, Opaque<Value>>,
}

#[derive(Default)]
pub struct ExternRefRegistry(Arc<Mutex<Entries>>);

impl ExternRefRegistry {
    fn entries(&self) -> MutexGuard<'_, Entries> {
        // Cleanup must not panic, including if another operation panicked.
        self.0.lock().unwrap_or_else(|error| error.into_inner())
    }

    pub fn allocate(&self, value: Value) -> Result<ExternRefHandle, Error> {
        let handle = {
            let mut entries = self.entries();
            entries.next_handle.checked_add(1).inspect(|&handle| {
                entries.next_handle = handle;
                entries.values.insert(handle, value.into());
            })
        };
        // Construct Ruby errors only after releasing the lock: allocation can
        // invoke the store's mark/compact hooks, which take the same lock.
        let handle = handle.ok_or_else(|| error!("externref handles exhausted"))?;
        Ok(ExternRefHandle {
            handle,
            registry: Arc::downgrade(&self.0),
        })
    }

    pub fn get(&self, ruby: &Ruby, handle: &ExternRefHandle) -> Result<Value, Error> {
        if !Weak::ptr_eq(&handle.registry, &Arc::downgrade(&self.0)) {
            return Err(error!("externref belongs to a different registry"));
        }
        let value = self.entries().values.get(&handle.handle).copied();
        value
            .map(|value| ruby.get_inner(value))
            .ok_or_else(|| error!("invalid externref handle"))
    }

    pub fn mark(&self, marker: &Marker) {
        for value in self.entries().values.values() {
            marker.mark_movable(*value);
        }
    }

    pub fn compact(&self, compactor: &Compactor) {
        for value in self.entries().values.values_mut() {
            *value = compactor.location(*value);
        }
    }
}

/// One registry entry per Wasmtime payload, not per copy of a rooted reference.
/// Dropping this payload can happen without the GVL, so it must never call Ruby.
/// A weak reference allows the store and its registry to be destroyed in either
/// order, without keeping Ruby values alive after the store is gone.
pub struct ExternRefHandle {
    handle: u64,
    registry: Weak<Mutex<Entries>>,
}

// We need this so values are garbage collected when they're no longer referenced by a table,
// global, or anything else that would hold a reference to it inside Wasmtime.
impl Drop for ExternRefHandle {
    fn drop(&mut self) {
        if let Some(registry) = self.registry.upgrade() {
            registry
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .values
                .remove(&self.handle);
        }
    }
}
