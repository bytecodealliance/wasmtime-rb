use super::{
    root,
    wasi_ctx_builder::{file_r, file_w, wasi_file},
    WasiCtxBuilder,
};
use crate::error;
use crate::helpers::OutputLimitedBuffer;
use magnus::{
    class, function, gc::Marker, method, prelude::*, typed_data::Obj, Error, Object, RString,
    RTypedData, Ruby, TypedData, Value,
};
use std::{borrow::Borrow, cell::RefCell, fs::File, path::PathBuf};
use wasmtime_wasi::preview1::WasiP1Ctx as WasiCtxImpl;

/// @yard
/// WASI context to be sent as {Store#new}’s +wasi_ctx+ keyword argument.
///
/// @see https://docs.rs/wasmtime-wasi/latest/wasmtime_wasi/struct.WasiCtx.html
///   Wasmtime's Rust doc
#[magnus::wrap(class = "Wasmtime::WasiCtx", size, free_immediately)]
pub struct WasiCtx {
    inner: RefCell<WasiCtxImpl>,
}

type RbSelf = Obj<WasiCtx>;

impl WasiCtx {
    pub fn from_inner(inner: WasiCtxImpl) -> Self {
        Self {
            inner: RefCell::new(inner),
        }
    }

    pub fn get_inner(&self) -> WasiCtxImpl {
        return self.inner.borrow().clone();
    }
}

pub fn init() -> Result<(), Error> {
    root().define_class("WasiCtx", class::object())?;
    Ok(())
}
