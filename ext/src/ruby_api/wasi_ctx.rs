use super::{root, WasiCtxBuilder, wasi_ctx_builder::{file_r, file_w, wasi_file}};
use crate::error;
use std::{fs::File, path::PathBuf, cell::RefCell, borrow::Borrow};
use magnus::{class, method, function, TypedData, typed_data::Obj, prelude::*, RTypedData, Error, Ruby, Value, RString, Object, gc::Marker};
use wasmtime_wasi::WasiCtx as WasiCtxImpl;
use deterministic_wasi_ctx::build_wasi_ctx as wasi_deterministic_ctx;
use wasi_common::pipe::ReadPipe;

#[magnus::wrap(class = "Wasmtime::WasiCtx", size, free_immediately)]
pub struct WasiCtx {
    inner: RefCell<WasiCtxImpl>,
}

type RbSelf = Obj<WasiCtx>;

impl WasiCtx {
    pub fn deterministic() -> Self {
        Self {inner: RefCell::new(wasi_deterministic_ctx()) }
    }

    pub fn from_inner(inner: WasiCtxImpl) -> Self {
        Self { inner: RefCell::new(inner) }
    }

    fn set_stdin_file(rb_self: RbSelf, path: RString) -> RbSelf {
        let inner = rb_self.inner.borrow_mut();
        let cs = file_r(path).map(wasi_file).unwrap();
        inner.set_stdin(cs);
        rb_self
    }
    fn set_stdin_string(rb_self: RbSelf, content: RString) -> RbSelf {
        let inner = rb_self.inner.borrow_mut();
        let str = unsafe {content.as_slice() };
        let pipe = ReadPipe::from(str);
        inner.set_stdin(Box::new(pipe));
        rb_self
    }
    fn set_stdout_file(rb_self: RbSelf, path: RString) -> RbSelf {
        let inner = rb_self.inner.borrow_mut();
        let cs = file_w(path).map(wasi_file).unwrap();
        inner.set_stdout(cs);
        rb_self
    }
    fn set_stderr_file(rb_self: RbSelf, path: RString) -> RbSelf {
        let inner = rb_self.inner.borrow_mut();
        let cs = file_w(path).map(wasi_file).unwrap();
        inner.set_stderr(cs);
        rb_self
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
