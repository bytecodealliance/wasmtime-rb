use crate::ruby_api::root;
use magnus::{error::ErrorType, value::Lazy, Error, ExceptionClass, Module, Ruby};
use std::borrow::Cow;

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

/// Utilities for reformatting error messages
pub trait ExceptionMessage {
    /// Append a message to an exception
    fn append<T>(self, extra: T) -> Self
    where
        T: Into<Cow<'static, str>>;
}

impl ExceptionMessage for magnus::Error {
    fn append<T>(self, extra: T) -> Self
    where
        T: Into<Cow<'static, str>>,
    {
        match self.error_type() {
            ErrorType::Error(class, msg) => Error::new(*class, format!("{}{}", msg, extra.into())),
            ErrorType::Exception(exception) => Error::new(
                exception.exception_class(),
                format!("{}{}", exception, extra.into()),
            ),
            _ => self,
        }
    }
}

pub(crate) fn missing_wasi_ctx_error() -> String {
    missing_wasi_error("WASI", "wasi_config")
}

pub(crate) fn missing_wasi_p1_ctx_error() -> String {
    missing_wasi_error("WASI p1", "wasi_p1_config")
}

fn missing_wasi_error(wasi_version: &str, option_name: &str) -> String {
    format!(
        "Store is missing {wasi_version} configuration.\n\n\
        When using `wasi: true`, the Store given to\n\
        `Linker#instantiate` must have a {wasi_version} configuration.\n\
        To fix this, provide the `wasi_config` when creating the Store:\n\
            Wasmtime::Store.new(engine, {option_name}: WasiConfig.new)"
    )
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
