use crate::ruby_api::root;
use magnus::rb_sys::value_from_raw;
use magnus::{exception::standard_error, memoize, ExceptionClass, Module};

/// Base error class for all Wasmtime errors.
pub fn base_error() -> ExceptionClass {
    *memoize!(ExceptionClass: root().define_error("Error", standard_error()).unwrap())
}

/// Ruby's `NotImplementedError` class.
pub fn not_implemented_error() -> ExceptionClass {
    *memoize!(ExceptionClass: {
        value_from_raw(rb_sys::rb_eNotImpError).try_convert().unwrap()
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
        Err(Error::new($crate::ruby_api::errors::not_implemented_error(), format!($($arg),*)))
    };
}
