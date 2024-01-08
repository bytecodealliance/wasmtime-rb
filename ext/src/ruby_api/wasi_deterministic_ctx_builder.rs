use super::root;
use crate::error;
use magnus::{
    class, function, gc, method, typed_data::Obj, DataTypeFunctions, Error, Module, Object, TypedData, RString
};
use std::cell::RefCell;
use std::{fs::File, path::PathBuf};
use wasi_common::pipe::ReadPipe;

enum ReadStream {
  Path(RString),
  String(RString),
}

impl ReadStream {
  pub fn mark(&self) {
      match self {
          Self::Path(s) => gc::mark(*s),
          Self::String(s) => gc::mark(*s),
      }
  }
}

enum WriteStream {
  Path(RString),
}
impl WriteStream {
  pub fn mark(&self) {
      match self {
          Self::Path(v) => gc::mark(*v),
      }
  }
}

#[derive(Default)]
struct WasiDeterministicCtxBuilderInner {
    stdin: Option<ReadStream>,
    stdout: Option<WriteStream>,
    stderr: Option<WriteStream>,
}

impl WasiDeterministicCtxBuilderInner {
    pub fn mark(&self) {
      if let Some(v) = self.stdin.as_ref() {
          v.mark();
      }
      if let Some(v) = self.stdout.as_ref() {
          v.mark();
      }
      if let Some(v) = self.stderr.as_ref() {
          v.mark();
      }
    }
}

/// @yard
/// WASI determinisitic context builder to be sent as {Store#new}’s +wasi_ctx+ keyword argument.
///
/// Instance methods mutate the current object and return +self+.
///
/// @see https://docs.rs/wasmtime-wasi/latest/wasmtime_wasi/sync/struct.WasiDeterministicCtxBuilder.html
///   Wasmtime's Rust doc
// #[derive(Debug)]
#[derive(Default, TypedData)]
#[magnus(class = "Wasmtime::WasiDeterministicCtxBuilder", size, mark, free_immediately)]
pub struct WasiDeterministicCtxBuilder {
    inner: RefCell<WasiDeterministicCtxBuilderInner>,
}

impl DataTypeFunctions for WasiDeterministicCtxBuilder {
    fn mark(&self) {
        self.inner.borrow().mark();
    }
}

type RbSelf = Obj<WasiDeterministicCtxBuilder>;

impl WasiDeterministicCtxBuilder {
    /// @yard
    /// Create a new {WasiDeterministicCtxBuilder}. By default, it has nothing: no stdin/out/err,
    /// no env, no argv, no file access.
    /// @return [WasiDeterministicCtxBuilder]
    pub fn new() -> Self {
        Self::default()
    }

     /// @yard
    /// Set stdin to read from the specified file.
    /// @param path [String] The path of the file to read from.
    /// @def set_stdin_file(path)
    /// @return [WasiCtxBuilder] +self+
    pub fn set_stdin_file(rb_self: RbSelf, path: RString) -> RbSelf {
      let mut inner = rb_self.get().inner.borrow_mut();
      inner.stdin = Some(ReadStream::Path(path));
      rb_self
    }

    /// @yard
    /// Set stdin to the specified String.
    /// @param content [String]
    /// @def set_stdin_string(content)
    /// @return [WasiCtxBuilder] +self+
    pub fn set_stdin_string(rb_self: RbSelf, content: RString) -> RbSelf {
      let mut inner = rb_self.get().inner.borrow_mut();
      inner.stdin = Some(ReadStream::String(content));
      rb_self
    }

    /// @yard
    /// Set stdout to write to a file. Will truncate the file if it exists,
    /// otherwise try to create it.
    /// @param path [String] The path of the file to write to.
    /// @def set_stdout_file(path)
    /// @return [WasiCtxBuilder] +self+
    pub fn set_stdout_file(rb_self: RbSelf, path: RString) -> RbSelf {
      let mut inner = rb_self.get().inner.borrow_mut();
      inner.stdout = Some(WriteStream::Path(path));
      rb_self
    }

    /// @yard
    /// Set stderr to write to a file. Will truncate the file if it exists,
    /// otherwise try to create it.
    /// @param path [String] The path of the file to write to.
    /// @def set_stderr_file(path)
    /// @return [WasiCtxBuilder] +self+
    pub fn set_stderr_file(rb_self: RbSelf, path: RString) -> RbSelf {
      let mut inner = rb_self.get().inner.borrow_mut();
      inner.stderr = Some(WriteStream::Path(path));
      rb_self
    }

    pub fn build_context(&self) -> Result<wasmtime_wasi::WasiCtx, Error> {
        let context = deterministic_wasi_ctx::build_wasi_ctx();
        let inner = self.inner.borrow();

        if let Some(stdin) = inner.stdin.as_ref() {
          match stdin {
              ReadStream::Path(path) => {
                // builder.stdin(file_r(*path).map(wasi_file)?)
                context.set_stdin(file_r(*path).map(wasi_file)?)
              },
              ReadStream::String(input) => {
                  // SAFETY: &[u8] copied before calling in to Ruby, no GC can happen before.
                  let pipe = ReadPipe::from(unsafe { input.as_slice() });
                  context.set_stdin(Box::new(pipe));
              }
          };
      }

      if let Some(stdout) = inner.stdout.as_ref() {
          match stdout {
              WriteStream::Path(path) => {
                context.set_stdout(file_w(*path).map(wasi_file)?)
              },
          };
      }

      if let Some(stderr) = inner.stderr.as_ref() {
        match stderr {
            WriteStream::Path(path) => {
              context.set_stdout(file_w(*path).map(wasi_file)?)
            },
        };
      }

      Ok(context)
    }
}

fn wasi_file(file: File) -> Box<wasi_cap_std_sync::file::File> {
    let file = cap_std::fs::File::from_std(file);
    let file = wasi_cap_std_sync::file::File::from_cap_std(file);
    Box::new(file)
}

fn file_r(path: RString) -> Result<File, Error> {
  // SAFETY: &str copied before calling in to Ruby, no GC can happen before.
  File::open(PathBuf::from(unsafe { path.as_str()? }))
      .map_err(|e| error!("Failed to open file {}\n{}", path, e))
}

fn file_w(path: RString) -> Result<File, Error> {
  // SAFETY: &str copied before calling in to Ruby, no GC can happen before.
  File::create(unsafe { path.as_str()? })
      .map_err(|e| error!("Failed to write to file {}\n{}", path, e))
}

pub fn init() -> Result<(), Error> {
    let class = root().define_class("WasiDeterministicCtxBuilder", class::object())?;
    class.define_singleton_method("new", function!(WasiDeterministicCtxBuilder::new, 0))?;

    class.define_method("set_stdin_file", method!(WasiDeterministicCtxBuilder::set_stdin_file, 1))?;
    class.define_method("set_stdin_string",method!(WasiDeterministicCtxBuilder::set_stdin_string, 1),)?;

    class.define_method("set_stdout_file",method!(WasiDeterministicCtxBuilder::set_stdout_file, 1),)?;

    class.define_method("set_stderr_file", method!(WasiDeterministicCtxBuilder::set_stderr_file, 1),)?;

    Ok(())
}
