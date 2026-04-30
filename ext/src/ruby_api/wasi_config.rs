use super::root;
use crate::error;
use crate::helpers::OutputLimitedBuffer;
use crate::ruby_api::convert::ToValType;
use crate::{define_rb_intern, helpers::SymbolEnum};
use lazy_static::lazy_static;
use magnus::block::Proc;
use magnus::value::ReprValue;
use magnus::{
    class, function, gc::Marker, method, typed_data::Obj, value::Opaque, DataTypeFunctions, Error,
    IntoValue, Module, Object, RArray, RHash, RString, Ruby, Symbol, TryConvert, TypedData, Value,
};
use rb_sys::ruby_rarray_flags::RARRAY_EMBED_FLAG;
use rb_sys::VALUE;
use std::cell::RefCell;
use std::convert::TryFrom;
use std::fs;
use std::future::Future;
use std::net::SocketAddr;
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;
use std::{fs::File, path::PathBuf};
use wasmtime_wasi::cli::{InputFile, OutputFile};
use wasmtime_wasi::p1::WasiP1Ctx;
use wasmtime_wasi::p2::pipe::MemoryInputPipe;
use wasmtime_wasi::sockets::SocketAddrUse;
use wasmtime_wasi::{DirPerms, FilePerms, WasiCtx, WasiCtxBuilder};

define_rb_intern!(
    READ => "read",
    WRITE => "write",
    MUTATE => "mutate",
    ALL => "all",
);

lazy_static! {
    static ref FILE_PERMS_MAPPING: SymbolEnum<'static, FilePerms> = {
        let mapping = vec![
            (*READ, FilePerms::READ),
            (*WRITE, FilePerms::WRITE),
            (*ALL, FilePerms::all()),
        ];

        SymbolEnum::new(":file_perms", mapping)
    };
    static ref DIR_PERMS_MAPPING: SymbolEnum<'static, DirPerms> = {
        let mapping = vec![
            (*READ, DirPerms::READ),
            (*MUTATE, DirPerms::MUTATE),
            (*ALL, DirPerms::all()),
        ];

        SymbolEnum::new(":dir_perms", mapping)
    };
}

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

struct PermsSymbolEnum(Symbol);

#[derive(Clone)]
struct MappedDirectory {
    host_path: Opaque<RString>,
    guest_path: Opaque<RString>,
    dir_perms: Opaque<Symbol>,
    file_perms: Opaque<Symbol>,
}
impl MappedDirectory {
    pub fn mark(&self, marker: &Marker) {
        marker.mark(self.host_path);
        marker.mark(self.guest_path);
        marker.mark(self.dir_perms);
        marker.mark(self.file_perms);
    }
}

struct SocketAddrProc {
    proc: Proc,
}

impl SocketAddrProc {
    fn call(&self, addr: SocketAddr, use_: SocketAddrUse) -> bool {
        let ruby = Ruby::get().unwrap();

        // Convert arguments to Ruby values
        let addr_str = ruby.str_new(&addr.to_string());
        let use_sym = socket_addr_use_to_symbol(&ruby, use_);

        match self.proc.call::<_, Value>((addr_str, use_sym)) {
            Ok(result) => bool::try_convert(result).unwrap_or(false),
            Err(_) => {
                // Exception in Ruby block, deny access
                false
            }
        }
    }
}

// SAFETY: We only access the Ruby proc when we have the GVL (during WASI operations).
// The Proc is kept alive by the Store's refs field, which is marked during GC.
unsafe impl Send for SocketAddrProc {}
unsafe impl Sync for SocketAddrProc {}

fn socket_addr_use_to_symbol(ruby: &Ruby, use_: SocketAddrUse) -> Symbol {
    match use_ {
        SocketAddrUse::TcpBind => ruby.to_symbol("tcp_bind"),
        SocketAddrUse::TcpConnect => ruby.to_symbol("tcp_connect"),
        SocketAddrUse::UdpBind => ruby.to_symbol("udp_bind"),
        SocketAddrUse::UdpConnect => ruby.to_symbol("udp_connect"),
        SocketAddrUse::UdpOutgoingDatagram => ruby.to_symbol("udp_outgoing_datagram"),
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
    mapped_directories: Vec<MappedDirectory>,
    inherit_network: bool,
    allow_tcp: Option<bool>,
    allow_udp: Option<bool>,
    allow_ip_name_lookup: Option<bool>,
    socket_addr_check: Option<Opaque<Proc>>,
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
        for v in &self.mapped_directories {
            v.mark(marker);
        }
        if let Some(v) = self.socket_addr_check.as_ref() {
            marker.mark(*v);
        }
    }
}

impl TryFrom<PermsSymbolEnum> for DirPerms {
    type Error = magnus::Error;
    fn try_from(value: PermsSymbolEnum) -> Result<Self, Error> {
        let ruby = Ruby::get_with(value.0);
        DIR_PERMS_MAPPING.get(value.0.into_value_with(&ruby))
    }
}

impl TryFrom<PermsSymbolEnum> for FilePerms {
    type Error = magnus::Error;
    fn try_from(value: PermsSymbolEnum) -> Result<Self, Error> {
        let ruby = Ruby::get_with(value.0);
        FILE_PERMS_MAPPING.get(value.0.into_value_with(&ruby))
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

    /// @yard
    /// Set mapped directory for host path and guest path.
    /// @param host_path [String]
    /// @param guest_path [String]
    /// @param dir_perms [Symbol] Directory permissions, one of :read, :mutate, or :all
    /// @param file_perms [Symbol] File permissions, one of :read, :write, or :all
    /// @def set_mapped_directory(host_path, guest_path, dir_perms, file_perms)
    /// @return [WasiConfig] +self+
    pub fn set_mapped_directory(
        rb_self: RbSelf,
        host_path: RString,
        guest_path: RString,
        dir_perms: Symbol,
        file_perms: Symbol,
    ) -> RbSelf {
        let mapped_dir = MappedDirectory {
            host_path: host_path.into(),
            guest_path: guest_path.into(),
            dir_perms: dir_perms.into(),
            file_perms: file_perms.into(),
        };

        let mut inner = rb_self.inner.borrow_mut();
        inner.mapped_directories.push(mapped_dir);

        rb_self
    }

    /// @yard
    /// Enable all network access by inheriting the host's network.
    /// This allows the WASI module to use TCP, UDP, and DNS resolution.
    ///
    /// Note: any network access happens while the Global VM Lock (GVL) is held, so other
    /// threads will be blocked in the meantime.
    /// @return [WasiConfig] +self+
    pub fn inherit_network(rb_self: RbSelf) -> RbSelf {
        let mut inner = rb_self.inner.borrow_mut();
        inner.inherit_network = true;
        rb_self
    }

    /// @yard
    /// Allow or deny TCP socket access. Allowed by default, can be used to blanket disable TCP.
    /// @param enabled [Boolean] Whether to allow TCP socket access
    /// @def allow_tcp(enabled)
    /// @return [WasiConfig] +self+
    pub fn allow_tcp(rb_self: RbSelf, enabled: bool) -> RbSelf {
        let mut inner = rb_self.inner.borrow_mut();
        inner.allow_tcp = Some(enabled);
        rb_self
    }

    /// @yard
    /// Allow or deny UDP socket access. Allowed by default, can be used to blanket disable UDP.
    /// @param enabled [Boolean] Whether to allow UDP socket access
    /// @def allow_udp(enabled)
    /// @return [WasiConfig] +self+
    pub fn allow_udp(rb_self: RbSelf, enabled: bool) -> RbSelf {
        let mut inner = rb_self.inner.borrow_mut();
        inner.allow_udp = Some(enabled);
        rb_self
    }

    /// @yard
    /// Allow or deny IP name lookup (DNS resolution).
    /// @param enabled [Boolean] Whether to allow IP name lookup
    /// @def allow_ip_name_lookup(enabled)
    /// @return [WasiConfig] +self+
    pub fn allow_ip_name_lookup(rb_self: RbSelf, enabled: bool) -> RbSelf {
        let mut inner = rb_self.inner.borrow_mut();
        inner.allow_ip_name_lookup = Some(enabled);
        rb_self
    }

    /// @yard
    /// Set a custom check function for socket address access control.
    /// The block will be called for each socket operation with the socket address (as a String)
    /// and the operation type (as a Symbol: :tcp_bind, :tcp_connect, :udp_bind, :udp_connect,
    /// :udp_outgoing_datagram).
    /// The block should return true to allow the operation or false to deny it.
    /// If the block raises an exception, the operation will be denied.
    ///
    /// Note: any network access happens while the Global VM Lock (GVL) is held, so other
    /// threads will be blocked in the meantime.
    ///
    /// @yieldparam addr [String] The socket address (e.g., "127.0.0.1:8080")
    /// @yieldparam use [Symbol] The type of socket operation
    /// @yieldreturn [Boolean] true to allow the operation, false to deny it
    /// @def socket_addr_check
    /// @return [WasiConfig] +self+
    pub fn socket_addr_check(ruby: &Ruby, rb_self: RbSelf) -> RbSelf {
        if ruby.block_given() {
            let proc = ruby.block_proc().unwrap();
            let mut inner = rb_self.inner.borrow_mut();
            inner.socket_addr_check = Some(proc.into());
        }
        rb_self
    }

    pub fn build_p1(&self, ruby: &Ruby) -> Result<(WasiP1Ctx, Option<Value>), Error> {
        let (mut builder, proc_value) = self.build_impl(ruby)?;
        let ctx = builder.build_p1();
        Ok((ctx, proc_value))
    }

    pub fn build(&self, ruby: &Ruby) -> Result<(WasiCtx, Option<Value>), Error> {
        let (mut builder, proc_value) = self.build_impl(ruby)?;
        let ctx = builder.build();
        Ok((ctx, proc_value))
    }

    fn build_impl(&self, ruby: &Ruby) -> Result<(WasiCtxBuilder, Option<Value>), Error> {
        let mut builder = WasiCtxBuilder::new();
        let inner = self.inner.borrow();
        let mut proc_to_retain = None;

        if let Some(stdin) = inner.stdin.as_ref() {
            match stdin {
                ReadStream::Inherit => builder.inherit_stdin(),
                ReadStream::Path(path) => {
                    builder.stdin(file_r(ruby.get_inner(*path)).map(InputFile::new)?)
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

        // Check for conflicting configuration: determinism and network access
        if inner.deterministic {
            let has_network_enabled = inner.inherit_network
                || inner.allow_tcp == Some(true)
                || inner.allow_udp == Some(true)
                || inner.allow_ip_name_lookup == Some(true);

            if has_network_enabled {
                return Err(error!(
                    "Cannot enable both determinism and network access. Deterministic mode requires network to be disabled."
                ));
            }

            deterministic_wasi_ctx::add_determinism_to_wasi_ctx_builder(&mut builder);
            // Explicitly disable network access in deterministic mode for defense-in-depth
            builder.allow_tcp(false);
            builder.allow_udp(false);
            builder.allow_ip_name_lookup(false);
        } else {
            // Apply network configuration
            if inner.inherit_network {
                builder.inherit_network();
            }
            if let Some(allow) = inner.allow_tcp {
                builder.allow_tcp(allow);
            }
            if let Some(allow) = inner.allow_udp {
                builder.allow_udp(allow);
            }
            if let Some(allow) = inner.allow_ip_name_lookup {
                builder.allow_ip_name_lookup(allow);
            }
        }

        if let Some(check_proc) = inner.socket_addr_check.as_ref() {
            let proc = ruby.get_inner(*check_proc);
            let socket_addr_proc = Arc::new(SocketAddrProc { proc });

            builder.socket_addr_check(move |addr, use_| {
                let socket_addr_proc = socket_addr_proc.clone();
                Box::pin(async move { socket_addr_proc.call(addr, use_) })
                    as Pin<Box<dyn Future<Output = bool> + Send + Sync>>
            });

            // Store the Proc as a Value so the Store can retain it
            proc_to_retain = Some(proc.as_value());
        }

        for mapped_dir in &inner.mapped_directories {
            let host_path = ruby.get_inner(mapped_dir.host_path).to_string()?;
            let guest_path = ruby.get_inner(mapped_dir.guest_path).to_string()?;
            let dir_perms = ruby.get_inner(mapped_dir.dir_perms);
            let file_perms = ruby.get_inner(mapped_dir.file_perms);

            builder
                .preopened_dir(
                    Path::new(&host_path),
                    &guest_path,
                    PermsSymbolEnum(dir_perms).try_into()?,
                    PermsSymbolEnum(file_perms).try_into()?,
                )
                .map_err(|e| error!("{}", e))?;
        }

        Ok((builder, proc_to_retain))
    }
}

pub fn file_r(path: RString) -> Result<File, Error> {
    // SAFETY: &str copied before calling in to Ruby, no GC can happen before.
    File::open(unsafe { path.as_str()? }).map_err(|e| error!("Failed to open file {}\n{}", path, e))
}

pub fn file_w(path: RString) -> Result<File, Error> {
    // SAFETY: &str copied before calling in to Ruby, no GC can happen before.
    File::create(unsafe { path.as_str()? })
        .map_err(|e| error!("Failed to write to file {}\n{}", path, e))
}

pub fn init(ruby: &Ruby) -> Result<(), Error> {
    let class = root().define_class("WasiConfig", ruby.class_object())?;
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

    class.define_method(
        "set_mapped_directory",
        method!(WasiConfig::set_mapped_directory, 4),
    )?;

    class.define_method("inherit_network", method!(WasiConfig::inherit_network, 0))?;
    class.define_method("allow_tcp", method!(WasiConfig::allow_tcp, 1))?;
    class.define_method("allow_udp", method!(WasiConfig::allow_udp, 1))?;
    class.define_method(
        "allow_ip_name_lookup",
        method!(WasiConfig::allow_ip_name_lookup, 1),
    )?;
    class.define_method(
        "socket_addr_check",
        method!(WasiConfig::socket_addr_check, 0),
    )?;

    Ok(())
}
