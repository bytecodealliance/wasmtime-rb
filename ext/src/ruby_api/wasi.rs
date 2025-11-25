use crate::{ruby_api::component, Linker};

use super::root;
use magnus::{class, function, typed_data::Obj, Error, Module, Object, RModule, Ruby};

#[magnus::wrap(class = "Wasmtime::WASI::P1", free_immediately)]
struct P1;

impl P1 {
    pub fn add_to_linker_sync(linker: Obj<Linker>) -> Result<(), Error> {
        linker.add_wasi_p1()
    }
}

#[magnus::wrap(class = "Wasmtime::WASI::P2", free_immediately)]
struct P2;

impl P2 {
    pub fn add_to_linker_sync(linker: Obj<component::Linker>) -> Result<(), Error> {
        linker.add_wasi_p2()
    }
}

pub fn init(ruby: &Ruby) -> Result<(), Error> {
    let namespace = root().define_module("WASI")?;

    let p1_class = namespace.define_class("P1", ruby.class_object())?;
    p1_class.define_singleton_method("add_to_linker_sync", function!(P1::add_to_linker_sync, 1))?;

    let p2_class = namespace.define_class("P2", ruby.class_object())?;
    p2_class.define_singleton_method("add_to_linker_sync", function!(P2::add_to_linker_sync, 1))?;

    Ok(())
}
