use magnus::{
    class, function, method, module::Module, typed_data::Obj, DataTypeFunctions, Error, Object,
    RModule, Ruby,
};
use wasmtime_wasi::p2::bindings::sync::Command;

use crate::{
    err, error,
    ruby_api::component::{linker::Linker, Component},
    Store,
};

#[magnus::wrap(class = "Wasmtime::Component::WasiCommand", size, free_immediately)]
pub struct WasiCommand {
    command: Command,
}

impl WasiCommand {
    /// @yard
    /// @def new(store, component, linker)
    /// @param store [Store]
    /// @param component [Component]
    /// @param linker [Linker]
    /// @return [WasiCommand]
    pub fn new(store: &Store, component: &Component, linker: &Linker) -> Result<Self, Error> {
        if linker.has_wasi() && !store.context().data().has_wasi_ctx() {
            return err!(
                "Store is missing WASI configuration.\n\n\
                When using `wasi: true`, the Store given to\n\
                `Linker#instantiate` must have a WASI configuration.\n\
                To fix this, provide the `wasi_config` when creating the Store:\n\
                    Wasmtime::Store.new(engine, wasi_config: WasiConfig.new)"
            );
        }
        let command =
            Command::instantiate(store.context_mut(), component.get(), &linker.inner_mut())
                .map_err(|e| error!("{e}"))?;
        Ok(Self { command })
    }

    /// @yard
    /// @def call_run(store)
    /// @param store [Store]
    /// @return [nil]
    pub fn call_run(_ruby: &Ruby, rb_self: Obj<Self>, store: &Store) -> Result<(), Error> {
        rb_self
            .command
            .wasi_cli_run()
            .call_run(store.context_mut())
            .map_err(|err| error!("{err}"))?
            .map_err(|_| error!("Error running `run`"))
    }
}

pub fn init(_ruby: &Ruby, namespace: &RModule) -> Result<(), Error> {
    let linker = namespace.define_class("WasiCommand", class::object())?;
    linker.define_singleton_method("new", function!(WasiCommand::new, 3))?;
    linker.define_method("call_run", method!(WasiCommand::call_run, 1))?;

    Ok(())
}
