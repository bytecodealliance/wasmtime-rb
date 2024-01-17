use super::{
    root,
    wasi_ctx_builder::{file_r, file_w, wasi_file},
    WasiCtxBuilder,
};
use crate::error;
use deterministic_wasi_ctx::build_wasi_ctx as wasi_deterministic_ctx;
use magnus::{
    class, function, gc::Marker, method, prelude::*, typed_data::Obj, Error, Object, RString,
    RTypedData, Ruby, TypedData, Value,
};
use std::{borrow::Borrow, cell::RefCell, fs::File, path::PathBuf};
use wasi_common::pipe::ReadPipe;
use wasmtime_wasi::WasiCtx as WasiCtxImpl;

/// @yard
/// WASI context to be sent as {Store#new}’s +wasi_ctx+ keyword argument.
///
/// Instance methods mutate the current object and return +self+.
///
/// @see https://docs.rs/wasmtime-wasi/latest/wasmtime_wasi/struct.WasiCtx.html
///   Wasmtime's Rust doc
#[magnus::wrap(class = "Wasmtime::WasiCtx", size, free_immediately)]
pub struct WasiCtx {
    inner: RefCell<WasiCtxImpl>,
}

type RbSelf = Obj<WasiCtx>;

impl WasiCtx {
    /// @yard
    /// Create a new deterministic {WasiCtx}. See https://github.com/Shopify/deterministic-wasi-ctx for more details
    /// @return [WasiCtx]
    pub fn deterministic() -> Self {
        Self {
            inner: RefCell::new(wasi_deterministic_ctx()),
        }
    }

    /// @yard
    /// Set stdin to read from the specified file.
    /// @param path [String] The path of the file to read from.
    /// @def set_stdin_file(path)
    /// @return [WasiCtxBuilder] +self+
    fn set_stdin_file(rb_self: RbSelf, path: RString) -> RbSelf {
        let inner = rb_self.inner.borrow_mut();
        let cs = file_r(path).map(wasi_file).unwrap();
        inner.set_stdin(cs);
        rb_self
    }

    /// @yard
    /// Set stdin to the specified String.
    /// @param content [String]
    /// @def set_stdin_string(content)
    /// @return [WasiCtx] +self+
    fn set_stdin_string(rb_self: RbSelf, content: RString) -> RbSelf {
        let inner = rb_self.inner.borrow_mut();
        let str = unsafe { content.as_slice() };
        let pipe = ReadPipe::from(str);
        inner.set_stdin(Box::new(pipe));
        rb_self
    }

    /// @yard
    /// Set stdout to write to a file. Will truncate the file if it exists,
    /// otherwise try to create it.
    /// @param path [String] The path of the file to write to.
    /// @def set_stdout_file(path)
    /// @return [WasiCtx] +self+
    fn set_stdout_file(rb_self: RbSelf, path: RString) -> RbSelf {
        let inner = rb_self.inner.borrow_mut();
        let cs = file_w(path).map(wasi_file).unwrap();
        inner.set_stdout(cs);
        rb_self
    }

    /// @yard
    /// Set stderr to write to a file. Will truncate the file if it exists,
    /// otherwise try to create it.
    /// @param path [String] The path of the file to write to.
    /// @def set_stderr_file(path)
    /// @return [WasiCtx] +self+
    fn set_stderr_file(rb_self: RbSelf, path: RString) -> RbSelf {
        let inner = rb_self.inner.borrow_mut();
        let cs = file_w(path).map(wasi_file).unwrap();
        inner.set_stderr(cs);
        rb_self
    }

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
    let class = root().define_class("WasiCtx", class::object())?;
    class.define_singleton_method("deterministic", function!(WasiCtx::deterministic, 0))?;
    class.define_method("set_stdin_file", method!(WasiCtx::set_stdin_file, 1))?;
    class.define_method("set_stdin_string", method!(WasiCtx::set_stdin_string, 1))?;
    class.define_method("set_stdout_file", method!(WasiCtx::set_stdout_file, 1))?;
    class.define_method("set_stderr_file", method!(WasiCtx::set_stderr_file, 1))?;
    Ok(())
}
