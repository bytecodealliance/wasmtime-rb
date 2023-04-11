use crate::ruby_api::root;
use magnus::{gc, Error};
use magnus::{memoize, ExceptionClass, Module};

/// Base error class for all Wasmtime errors.
pub fn base_error() -> ExceptionClass {
    *memoize!(ExceptionClass: {
        let err = root().const_get("Error").unwrap();
        gc::register_mark_object(err);
        err
    })
}

/// Raised when failing to convert the return value of a Ruby-backed Func to
/// Wasm types.
pub fn result_error() -> ExceptionClass {
    *memoize!(ExceptionClass: {
        let err = root().const_get("ResultError").unwrap();
        gc::register_mark_object(err);
        err
    })
}

/// Raised when converting an {Extern} to its concrete type fails.
pub fn conversion_error() -> ExceptionClass {
    *memoize!(ExceptionClass: {
        let err = root().const_get("ConversionError").unwrap();
        gc::register_mark_object(err);
        err
    })
}

/// Raised when a WASI program terminates early by calling +exit+.
pub fn wasi_exit_error() -> ExceptionClass {
    *memoize!(ExceptionClass: {
        let err = root().const_get("WasiExit").unwrap();
        gc::register_mark_object(err);
        err
    })
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
