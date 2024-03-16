use crate::ruby_api::root;
use magnus::{value::Lazy, Error, ExceptionClass, Module, Ruby};

/// Base error class for all Wasmtime errors.
pub fn base_error() -> ExceptionClass {
    static ERR: Lazy<ExceptionClass> = Lazy::new(|_| root().const_get("Error").unwrap());
    let ruby = Ruby::get().unwrap();
    ruby.get_inner(&ERR)
}

/// Raised when failing to convert the return value of a Ruby-backed Func to
/// Wasm types.
pub fn result_error() -> ExceptionClass {
    static ERR: Lazy<ExceptionClass> = Lazy::new(|_| root().const_get("ResultError").unwrap());
    let ruby = Ruby::get().unwrap();
    ruby.get_inner(&ERR)
}

/// Raised when converting an {Extern} to its concrete type fails.
pub fn conversion_error() -> ExceptionClass {
    static ERR: Lazy<ExceptionClass> = Lazy::new(|_| root().const_get("ConversionError").unwrap());
    let ruby = Ruby::get().unwrap();
    ruby.get_inner(&ERR)
}

/// Raised when attempting to use a module that has been disposed.
pub fn module_disposed_error() -> ExceptionClass {
    static ERR: Lazy<ExceptionClass> =
        Lazy::new(|_| root().const_get("ModuleDisposedError").unwrap());
    let ruby = Ruby::get().unwrap();
    ruby.get_inner(&ERR)
}

pub fn module_disposed_err<T>() -> Result<T, Error> {
    Err(Error::new(
        module_disposed_error(),
        "module has been disposed",
    ))
}

/// Raised when a WASI program terminates early by calling +exit+.
pub fn wasi_exit_error() -> ExceptionClass {
    static ERR: Lazy<ExceptionClass> = Lazy::new(|_| root().const_get("WasiExit").unwrap());
    let ruby = Ruby::get().unwrap();
    ruby.get_inner(&ERR)
}

#[macro_export]
macro_rules! err {
    ($($arg:expr),*) => {
        Result::Err($crate::error!($($arg),*))
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:expr),*) => {
        Error::new($crate::ruby_api::errors::base_error(), format!($($arg),*))
    };
}

#[macro_export]
macro_rules! not_implemented {
    ($($arg:expr),*) => {
        Err(Error::new(magnus::exception::not_imp_error(), format!($($arg),*)))
    };
}

#[macro_export]
macro_rules! conversion_err {
    ($($arg:expr),*) => {
        Err(Error::new($crate::ruby_api::errors::conversion_error(), format!("cannot convert {} to {}", $($arg),*)))
    };
}

mod bundled {
    include!(concat!(env!("OUT_DIR"), "/bundled/error.rs"));
}

pub fn init() -> Result<(), Error> {
    bundled::init()?;

    let _ = base_error();
    let _ = result_error();
    let _ = conversion_error();
    let _ = wasi_exit_error();

    Ok(())
}
