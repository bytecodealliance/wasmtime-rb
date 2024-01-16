use super::{root, WasiCtxBuilder, wasi_ctx_builder::{file_r, file_w, wasi_file}};
use crate::error;
use std::{fs::File, path::PathBuf, cell::RefCell};
use magnus::{class, method, function, prelude::*, RTypedData, Error, Ruby, Value, RString};
use wasmtime_wasi::WasiCtx as WasiCtxImpl;
use deterministic_wasi_ctx::build_wasi_ctx as wasi_deterministic_ctx;
use wasi_common::pipe::ReadPipe;

#[magnus::wrap(class = "Wasmtime::WasiCtx", size, free_immediately)]
pub struct WasiCtx {
    inner: RefCell<WasiCtxImpl>,
}

impl WasiCtx {

    // pub fn from_builder(builder: WasiCtxBuilder) -> Self {
        // builder.build_context()
        // Self { inner: RefCell::new(WasiCtxImpl) }
    // }

    fn deterministic() -> Self {
        Self {inner: RefCell::new(wasi_deterministic_ctx()) }
    }

    fn set_stdin_file(&self, path: RString) {
        let inner = self.inner.borrow_mut();
        let cs = file_r(path).map(wasi_file).unwrap();
        inner.set_stdin(cs);
    }
    fn set_stdin_string(&self, content: RString) {
        let inner = self.inner.borrow_mut();
        let str = unsafe {content.as_slice() };
        let pipe = ReadPipe::from(str);
        inner.set_stdin(Box::new(pipe));
    }
    fn set_stdout_file(&self, path: RString) {
        let inner = self.inner.borrow_mut();
        let cs = file_w(path).map(wasi_file).unwrap();
        inner.set_stdout(cs);
    }
    fn set_stderr_file(&self, path: RString) {
        let inner = self.inner.borrow_mut();
        let cs = file_w(path).map(wasi_file).unwrap();
        inner.set_stderr(cs);
    }

    pub fn get_inner(&self) -> WasiCtxImpl {
        return self.inner.borrow().clone();
    }
}

pub fn init() -> Result<(), Error> {
    let class = root().define_class("WasiCtx", class::object())?;
    class.define_singleton_method("deterministic", function!(WasiCtx::deterministic, 0))?;
    class.define_method("set_stdin_file", method!(WasiCtx::set_stdin_file, 1))?;
    class.define_method("set_stdin_string", method!(WasiCtx::set_stdin_string, 1))?;
    class.define_method("set_stdout_file", method!(WasiCtx::set_stdout_file, 1))?;
    class.define_method("set_stderr_file", method!(WasiCtx::set_stderr_file, 1))?;
    Ok(())
}
