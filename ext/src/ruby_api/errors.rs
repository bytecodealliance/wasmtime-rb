use crate::ruby_api::root;
use magnus::rb_sys::FromRawValue;
use magnus::{exception::standard_error, memoize, ExceptionClass, Module};
use magnus::{gc, method, Attr, Error, Object, RObject, StaticSymbol, Value};

/// @yard
/// @rename Wasmtime::Error
/// Base error class for all Wasmtime errors.
pub fn base_error() -> ExceptionClass {
    *memoize!(ExceptionClass: {
        let err = root().define_error("Error", standard_error()).unwrap();
        gc::register_mark_object(err);
        err
    })
}

/// @yard
/// @rename Wasmtime::ResultError
/// Raised when failing to convert the return value of a Ruby-backed Func to
/// Wasm types.
pub fn result_error() -> ExceptionClass {
    *memoize!(ExceptionClass: {
        let err = root().define_error("ResultError", base_error()).unwrap();
        gc::register_mark_object(err);
        err
    })
}

/// @yard
/// @rename Wasmtime::ConversionError
/// Raised when converting an {Extern} to its concrete type fails.
pub fn conversion_error() -> ExceptionClass {
    *memoize!(ExceptionClass: {
        let err = root().define_error("ConversionError", base_error()).unwrap();
        gc::register_mark_object(err);
        err
    })
}

/// Ruby's `NotImplementedError` class.
pub fn not_implemented_error() -> ExceptionClass {
    *memoize!(ExceptionClass: {
        Value::from_raw(rb_sys::rb_eNotImpError).try_convert().unwrap()
    })
}

/// @yard
/// @rename Wasmtime::WasiExit
/// Raised when a WASI program terminates early by calling +exit+.
pub fn wasi_exit_error() -> ExceptionClass {
    *memoize!(ExceptionClass: {
        let err = root().define_error("WasiExit", base_error()).unwrap();

        gc::register_mark_object(err);

        err.define_attr("code", Attr::Read).unwrap();

        fn initialize(rb_self: RObject, code: Value) -> Result<(), Error> {
            rb_self.ivar_set("@code", code)?;
            Ok(())
        }
        err.define_method("initialize", method!(initialize, 1)).unwrap();

        fn message(rb_self: RObject) -> Result<String, Error> {
            let code: Value = rb_self.ivar_get("@code")?;
            Ok(format!("WASI exit with code {code}"))
        }
        err.define_method("message", method!(message, 0)).unwrap();

        err
    })
}

/// @yard
/// @rename Wasmtime::Trap
/// Raised on Wasm trap.
pub fn trap_error() -> ExceptionClass {
    *memoize!(ExceptionClass: {
        let err = root().define_error("Trap", base_error()).unwrap();

        gc::register_mark_object(err);

        err.const_set("STACK_OVERFLOW", StaticSymbol::new("stack_overflow")).unwrap();
        err.const_set("MEMORY_OUT_OF_BOUNDS", StaticSymbol::new("memory_out_of_bounds")).unwrap();
        err.const_set("HEAP_MISALIGNED", StaticSymbol::new("heap_misaligned")).unwrap();
        err.const_set("TABLE_OUT_OF_BOUNDS", StaticSymbol::new("table_out_of_bounds")).unwrap();
        err.const_set("INDIRECT_CALL_TO_NULL", StaticSymbol::new("indirect_call_to_null")).unwrap();
        err.const_set("BAD_SIGNATURE", StaticSymbol::new("bad_signature")).unwrap();
        err.const_set("INTEGER_OVERFLOW", StaticSymbol::new("integer_overflow")).unwrap();
        err.const_set("INTEGER_DIVISION_BY_ZERO", StaticSymbol::new("integer_division_by_zero")).unwrap();
        err.const_set("BAD_CONVERSION_TO_INTEGER", StaticSymbol::new("bad_conversion_to_integer")).unwrap();
        err.const_set("UNREACHABLE_CODE_REACHED", StaticSymbol::new("unreachable_code_reached")).unwrap();
        err.const_set("INTERRUPT", StaticSymbol::new("interrupt")).unwrap();
        err.const_set("ALWAYS_TRAP_ADAPTER", StaticSymbol::new("always_trap_adapter")).unwrap();
        err.const_set("OUT_OF_FUEL", StaticSymbol::new("out_of_fuel")).unwrap();
        err.const_set("UNKNOWN", StaticSymbol::new("unknown")).unwrap();

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
        Err(Error::new($crate::ruby_api::errors::not_implemented_error(), format!($($arg),*)))
    };
}

#[macro_export]
macro_rules! conversion_err {
    ($($arg:expr),*) => {
        Err(Error::new($crate::ruby_api::errors::conversion_error(), format!("cannot convert {} to {}", $($arg),*)))
    };
}

pub fn init() -> Result<(), Error> {
    let _ = base_error();
    let _ = result_error();
    let _ = conversion_error();
    let _ = wasi_exit_error();
    let _ = trap_error();

    Ok(())
}
