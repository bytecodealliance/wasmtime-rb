use super::store::Store;
use crate::ruby_api::{root, trap::Trap};
use magnus::rb_sys::FromRawValue;
use magnus::{exception::standard_error, memoize, ExceptionClass, Module};
use magnus::{Error, Value};

/// Base error class for all Wasmtime errors.
pub fn base_error() -> ExceptionClass {
    *memoize!(ExceptionClass: root().define_error("Error", standard_error()).unwrap())
}

/// Ruby's `NotImplementedError` class.
pub fn not_implemented_error() -> ExceptionClass {
    *memoize!(ExceptionClass: {
        Value::from_raw(rb_sys::rb_eNotImpError).try_convert().unwrap()
    })
}

/// The `Wasmtime::ConversionError` class.
pub fn conversion_error() -> ExceptionClass {
    *memoize!(ExceptionClass: root().define_error("ConversionError", base_error()).unwrap())
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
        Err(Error::new($crate::ruby_api::errors::not_implemented_error(), format!($($arg),*)))
    };
}

#[macro_export]
macro_rules! conversion_err {
    ($($arg:expr),*) => {
        Err(Error::new($crate::ruby_api::errors::conversion_error(), format!("cannot convert {} to {}", $($arg),*)))
    };
}

pub fn handle_wasm_error(store: &Store, error: anyhow::Error) -> Error {
    store
        .context_mut()
        .data_mut()
        .take_last_error()
        .unwrap_or_else(|| match error.downcast_ref::<wasmtime::Trap>() {
            Some(t) => Trap::from(t.to_owned()).into(),
            _ => error!("{}", error),
        })
}

pub fn init() -> Result<(), Error> {
    let _ = base_error();
    let _ = conversion_error();

    Ok(())
}
