use crate::ruby_api::root;
use magnus::{
    exception::standard_error, gc::register_mark_object, memoize, ExceptionClass, Module,
};

/// Base error class for all Wasmtime errors.
pub fn base_error() -> ExceptionClass {
    *memoize!(ExceptionClass: {
        let exc = root().define_error("Error", standard_error()).unwrap();
        register_mark_object(exc);
        exc
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
