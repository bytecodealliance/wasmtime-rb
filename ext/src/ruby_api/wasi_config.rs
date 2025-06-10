use super::root;
use crate::error;
use crate::helpers::OutputLimitedBuffer;
use magnus::{
    class, function, gc::Marker, method, typed_data::Obj, value::Opaque, DataTypeFunctions, Error,
    Module, Object, RArray, RHash, RString, Ruby, TryConvert, TypedData,
};
use std::cell::RefCell;
use std::fs;
use std::{fs::File, path::PathBuf};
use wasmtime_wasi::p2::pipe::MemoryInputPipe;
use wasmtime_wasi::p2::{OutputFile, WasiCtxBuilder};
use wasmtime_wasi::preview1::WasiP1Ctx;

enum ReadStream {
    Inherit,
    Path(Opaque<RString>),
    String(Opaque<RString>),
}

impl ReadStream {
    pub fn mark(&self, marker: &Marker) {
        match self {
            Self::Inherit => (),
            Self::Path(s) => marker.mark(*s),
            Self::String(s) => marker.mark(*s),
        }
    }
}

enum WriteStream {
    Inherit,
    Path(Opaque<RString>),
    Buffer(Opaque<RString>, usize),
}
impl WriteStream {
    pub fn mark(&self, marker: &Marker) {
        match self {
            Self::Inherit => (),
            Self::Path(v) => marker.mark(*v),
            Self::Buffer(v, _) => marker.mark(*v),
        }
    }
}

#[derive(Default)]
struct WasiConfigInner {
    stdin: Option<ReadStream>,
    stdout: Option<WriteStream>,
    stderr: Option<WriteStream>,
    env: Option<Opaque<RHash>>,
    args: Option<Opaque<RArray>>,
    deterministic: bool,
}

impl WasiConfigInner {
    pub fn mark(&self, marker: &Marker) {
        if let Some(v) = self.stdin.as_ref() {
            v.mark(marker);
        }
        if let Some(v) = self.stdout.as_ref() {
            v.mark(marker);
        }
        if let Some(v) = self.stderr.as_ref() {
            v.mark(marker);
        }
        if let Some(v) = self.env.as_ref() {
            marker.mark(*v);
        }
        if let Some(v) = self.args.as_ref() {
            marker.mark(*v);
        }
    }
}

/// @yard
/// WASI config to be sent as {Store#new}’s +wasi_config+ keyword argument.
///
/// Instance methods mutate the current object and return +self+.
///
// #[derive(Debug)]
#[derive(Default, TypedData)]
#[magnus(class = "Wasmtime::WasiConfig", size, mark, free_immediately)]
pub struct WasiConfig {
    inner: RefCell<WasiConfigInner>,
}

impl DataTypeFunctions for WasiConfig {
    fn mark(&self, marker: &Marker) {
        self.inner.borrow().mark(marker);
    }
}

type RbSelf = Obj<WasiConfig>;

impl WasiConfig {
    /// @yard
    /// Create a new {WasiConfig}. By default, it has nothing: no stdin/out/err,
    /// no env, no argv, no file access.
    /// @return [WasiConfig]
    pub fn new() -> Self {
        Self::default()
    }

    /// @yard
    /// Use deterministic implementations for clocks and random methods.
    /// @return [WasiConfig] +self+
    pub fn add_determinism(rb_self: RbSelf) -> RbSelf {
        let mut inner = rb_self.inner.borrow_mut();
        inner.deterministic = true;
        rb_self
    }

    /// @yard
    /// Inherit stdin from the current Ruby process.
    /// @return [WasiConfig] +self+
    pub fn inherit_stdin(rb_self: RbSelf) -> RbSelf {
        let mut inner = rb_self.inner.borrow_mut();
        inner.stdin = Some(ReadStream::Inherit);
        rb_self
    }

    /// @yard
    /// Set stdin to read from the specified file.
    /// @param path [String] The path of the file to read from.
    /// @def set_stdin_file(path)
    /// @return [WasiConfig] +self+
    pub fn set_stdin_file(rb_self: RbSelf, path: RString) -> RbSelf {
        let mut inner = rb_self.inner.borrow_mut();
        inner.stdin = Some(ReadStream::Path(path.into()));
        rb_self
    }

    /// @yard
    /// Set stdin to the specified String.
    /// @param content [String]
    /// @def set_stdin_string(content)
    /// @return [WasiConfig] +self+
    pub fn set_stdin_string(rb_self: RbSelf, content: RString) -> RbSelf {
        let mut inner = rb_self.inner.borrow_mut();
        inner.stdin = Some(ReadStream::String(content.into()));
        rb_self
    }

    /// @yard
    /// Inherit stdout from the current Ruby process.
    /// @return [WasiConfig] +self+
    pub fn inherit_stdout(rb_self: RbSelf) -> RbSelf {
        let mut inner = rb_self.inner.borrow_mut();
        inner.stdout = Some(WriteStream::Inherit);
        rb_self
    }

    /// @yard
    /// Set stdout to write to a file. Will truncate the file if it exists,
    /// otherwise try to create it.
    /// @param path [String] The path of the file to write to.
    /// @def set_stdout_file(path)
    /// @return [WasiConfig] +self+
    pub fn set_stdout_file(rb_self: RbSelf, path: RString) -> RbSelf {
        let mut inner = rb_self.inner.borrow_mut();
        inner.stdout = Some(WriteStream::Path(path.into()));
        rb_self
    }

    /// @yard
    /// Set stdout to write to a string buffer.
    /// If the string buffer is frozen, Wasm execution will raise a Wasmtime::Error error.
    /// No encoding checks are done on the resulting string, it is the caller's responsibility to ensure the string contains a valid encoding
    /// @param buffer [String] The string buffer to write to.
    /// @param capacity [Integer] The maximum number of bytes that can be written to the output buffer.
    /// @def set_stdout_buffer(buffer, capacity)
    /// @return [WasiConfig] +self+
    pub fn set_stdout_buffer(rb_self: RbSelf, buffer: RString, capacity: usize) -> RbSelf {
        let mut inner = rb_self.inner.borrow_mut();
        inner.stdout = Some(WriteStream::Buffer(buffer.into(), capacity));
        rb_self
    }

    /// @yard
    /// Inherit stderr from the current Ruby process.
    /// @return [WasiConfig] +self+
    pub fn inherit_stderr(rb_self: RbSelf) -> RbSelf {
        let mut inner = rb_self.inner.borrow_mut();
        inner.stderr = Some(WriteStream::Inherit);
        rb_self
    }

    /// @yard
    /// Set stderr to write to a file. Will truncate the file if it exists,
    /// otherwise try to create it.
    /// @param path [String] The path of the file to write to.
    /// @def set_stderr_file(path)
    /// @return [WasiConfig] +self+
    pub fn set_stderr_file(rb_self: RbSelf, path: RString) -> RbSelf {
        let mut inner = rb_self.inner.borrow_mut();
        inner.stderr = Some(WriteStream::Path(path.into()));
        rb_self
    }

    /// @yard
    /// Set stderr to write to a string buffer.
    /// If the string buffer is frozen, Wasm execution will raise a Wasmtime::Error error.
    /// No encoding checks are done on the resulting string, it is the caller's responsibility to ensure the string contains a valid encoding
    /// @param buffer [String] The string buffer to write to.
    /// @param capacity [Integer] The maximum number of bytes that can be written to the output buffer.
    /// @def set_stderr_buffer(buffer, capacity)
    /// @return [WasiConfig] +self+
    pub fn set_stderr_buffer(rb_self: RbSelf, buffer: RString, capacity: usize) -> RbSelf {
        let mut inner = rb_self.inner.borrow_mut();
        inner.stderr = Some(WriteStream::Buffer(buffer.into(), capacity));
        rb_self
    }
    /// @yard
    /// Set env to the specified +Hash+.
    /// @param env [Hash<String, String>]
    /// @def set_env(env)
    /// @return [WasiConfig] +self+
    pub fn set_env(rb_self: RbSelf, env: RHash) -> RbSelf {
        let mut inner = rb_self.inner.borrow_mut();
        inner.env = Some(env.into());
        rb_self
    }

    /// @yard
    /// Set the arguments (argv) to the specified +Array+.
    /// @param args [Array<String>]
    /// @def set_argv(args)
    /// @return [WasiConfig] +self+
    pub fn set_argv(rb_self: RbSelf, argv: RArray) -> RbSelf {
        let mut inner = rb_self.inner.borrow_mut();
        inner.args = Some(argv.into());
        rb_self
    }

    pub fn build(&self, ruby: &Ruby) -> Result<WasiP1Ctx, Error> {
        let mut builder = WasiCtxBuilder::new();
        let inner = self.inner.borrow();

        if let Some(stdin) = inner.stdin.as_ref() {
            match stdin {
                ReadStream::Inherit => builder.inherit_stdin(),
                ReadStream::Path(path) => {
                    // Reading the whole file into memory and passing it as an
                    // in-memory buffer because I cannot find a public struct
                    // to use a file as an input that implements `StdinStream`
                    // and the implementation would not be trivial.
                    // TODO: Use
                    // https://github.com/bytecodealliance/wasmtime/pull/10968
                    // when it's in a published version.
                    // SAFETY: &[u8] copied before calling in to Ruby, no GC can happen before.
                    let inner = ruby.get_inner(*path);
                    let path = PathBuf::from(unsafe { inner.as_str() }?);
                    let contents = fs::read(path).map_err(|e| error!("{e}"))?;
                    builder.stdin(MemoryInputPipe::new(contents))
                }
                ReadStream::String(input) => {
                    // SAFETY: &[u8] copied before calling in to Ruby, no GC can happen before.
                    let inner = ruby.get_inner(*input);
                    builder.stdin(MemoryInputPipe::new(unsafe { inner.as_slice() }.to_vec()))
                }
            };
        }

        if let Some(stdout) = inner.stdout.as_ref() {
            match stdout {
                WriteStream::Inherit => builder.inherit_stdout(),
                WriteStream::Path(path) => {
                    builder.stdout(file_w(ruby.get_inner(*path)).map(OutputFile::new)?)
                }
                WriteStream::Buffer(buffer, capacity) => {
                    builder.stdout(OutputLimitedBuffer::new(*buffer, *capacity))
                }
            };
        }

        if let Some(stderr) = inner.stderr.as_ref() {
            match stderr {
                WriteStream::Inherit => builder.inherit_stderr(),
                WriteStream::Path(path) => {
                    builder.stderr(file_w(ruby.get_inner(*path)).map(OutputFile::new)?)
                }
                WriteStream::Buffer(buffer, capacity) => {
                    builder.stderr(OutputLimitedBuffer::new(*buffer, *capacity))
                }
            };
        }

        if let Some(args) = inner.args.as_ref() {
            // SAFETY: no gc can happen nor do we write to `args`.
            for item in unsafe { ruby.get_inner(*args).as_slice() } {
                let arg = RString::try_convert(*item)?;
                // SAFETY: &str copied before calling in to Ruby, no GC can happen before.
                let arg = unsafe { arg.as_str() }?;
                builder.arg(arg);
            }
        }

        if let Some(env_hash) = inner.env.as_ref() {
            let env_vec: Vec<(String, String)> = ruby.get_inner(*env_hash).to_vec()?;
            builder.envs(&env_vec);
        }

        if inner.deterministic {
            deterministic_wasi_ctx::add_determinism_to_wasi_ctx_builder(&mut builder);
        }

        let ctx = builder.build_p1();
        Ok(ctx)
    }
}

pub fn file_w(path: RString) -> Result<File, Error> {
    // SAFETY: &str copied before calling in to Ruby, no GC can happen before.
    File::create(unsafe { path.as_str()? })
        .map_err(|e| error!("Failed to write to file {}\n{}", path, e))
}

pub fn init() -> Result<(), Error> {
    let class = root().define_class("WasiConfig", class::object())?;
    class.define_singleton_method("new", function!(WasiConfig::new, 0))?;

    class.define_method("add_determinism", method!(WasiConfig::add_determinism, 0))?;

    class.define_method("inherit_stdin", method!(WasiConfig::inherit_stdin, 0))?;
    class.define_method("set_stdin_file", method!(WasiConfig::set_stdin_file, 1))?;
    class.define_method("set_stdin_string", method!(WasiConfig::set_stdin_string, 1))?;

    class.define_method("inherit_stdout", method!(WasiConfig::inherit_stdout, 0))?;
    class.define_method("set_stdout_file", method!(WasiConfig::set_stdout_file, 1))?;
    class.define_method(
        "set_stdout_buffer",
        method!(WasiConfig::set_stdout_buffer, 2),
    )?;

    class.define_method("inherit_stderr", method!(WasiConfig::inherit_stderr, 0))?;
    class.define_method("set_stderr_file", method!(WasiConfig::set_stderr_file, 1))?;
    class.define_method(
        "set_stderr_buffer",
        method!(WasiConfig::set_stderr_buffer, 2),
    )?;

    class.define_method("set_env", method!(WasiConfig::set_env, 1))?;

    class.define_method("set_argv", method!(WasiConfig::set_argv, 1))?;

    Ok(())
}
