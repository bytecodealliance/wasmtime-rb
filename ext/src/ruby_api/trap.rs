use std::convert::TryFrom;

use crate::helpers::WrappedStruct;
use crate::ruby_api::{errors::base_error, root};
use magnus::Error;
use magnus::{
    memoize, method, rb_sys::AsRawValue, DataTypeFunctions, ExceptionClass, Module as _, Symbol,
    TypedData, Value,
};

pub fn trap_error() -> ExceptionClass {
    *memoize!(ExceptionClass: root().define_error("Trap", base_error()).unwrap())
}

macro_rules! trap_const {
    ($trap:ident) => {
        trap_error().const_get(stringify!($trap)).map(Some)
    };
}

#[derive(TypedData, Debug)]
#[magnus(class = "Wasmtime::Trap", size, free_immediatly)]
/// @yard
pub struct Trap {
    trap: wasmtime::Trap,
    wasm_backtrace: Option<wasmtime::WasmBacktrace>,
}
impl DataTypeFunctions for Trap {}

impl Trap {
    pub fn new(trap: wasmtime::Trap, wasm_backtrace: Option<wasmtime::WasmBacktrace>) -> Self {
        Self {
            trap,
            wasm_backtrace,
        }
    }

    /// @yard
    /// Returns a textual description of the trap error, for example:
    ///     wasm trap: wasm `unreachable` instruction executed
    /// @return [String]
    pub fn message(&self) -> String {
        self.trap.to_string()
    }

    /// @yard
    /// Returns a textual representation of the Wasm backtrce, if it exists.
    /// For example:
    ///    error while executing at wasm backtrace:
    ///        0:   0x1a - <unknown>!<wasm function 0>
    /// @return [String, nil]
    pub fn wasm_backtrace_message(&self) -> Option<String> {
        self.wasm_backtrace.as_ref().map(|bt| format!("{}", bt))
    }

    /// @yard
    /// Returns the trap code as a Symbol, possibly nil if the trap did not
    /// origin from Wasm code. All possible trap codes are defined as constants on {Trap}.
    /// @return [Symbol, nil]
    pub fn code(&self) -> Result<Option<Symbol>, Error> {
        match self.trap {
            wasmtime::Trap::HeapMisaligned => trap_const!(HEAP_MISALIGNED),
            wasmtime::Trap::TableOutOfBounds => trap_const!(TABLE_OUT_OF_BOUNDS),
            wasmtime::Trap::IndirectCallToNull => trap_const!(INDIRECT_CALL_TO_NULL),
            wasmtime::Trap::BadSignature => trap_const!(BAD_SIGNATURE),
            wasmtime::Trap::IntegerOverflow => trap_const!(INTEGER_OVERFLOW),
            wasmtime::Trap::IntegerDivisionByZero => trap_const!(INTEGER_DIVISION_BY_ZERO),
            wasmtime::Trap::BadConversionToInteger => trap_const!(BAD_CONVERSION_TO_INTEGER),
            wasmtime::Trap::UnreachableCodeReached => trap_const!(UNREACHABLE_CODE_REACHED),
            wasmtime::Trap::Interrupt => trap_const!(INTERRUPT),
            wasmtime::Trap::AlwaysTrapAdapter => trap_const!(ALWAYS_TRAP_ADAPTER),
            wasmtime::Trap::OutOfFuel => trap_const!(OUT_OF_FUEL),
            // When adding a trap code here, define a matching constant on Wasmtime::Trap (in Ruby)
            _ => trap_const!(UNKNOWN),
        }
    }

    pub fn inspect(rb_self: WrappedStruct<Self>) -> Result<String, Error> {
        let rs_self = rb_self.get()?;

        Ok(format!(
            "#<Wasmtime::Trap:0x{:016x} @trap_code={}>",
            rb_self.to_value().as_raw(),
            Value::from(rs_self.code()?).inspect()
        ))
    }
}

impl From<Trap> for Error {
    fn from(trap: Trap) -> Self {
        magnus::Value::from(trap)
            .try_convert::<magnus::Exception>()
            .unwrap() // Can't fail: Wasmtime::Trap is an Exception
            .into()
    }
}

impl TryFrom<anyhow::Error> for Trap {
    type Error = anyhow::Error;

    fn try_from(value: anyhow::Error) -> Result<Self, Self::Error> {
        match value.downcast_ref::<wasmtime::Trap>() {
            Some(trap) => {
                let trap = trap.to_owned();
                let bt = value.downcast::<wasmtime::WasmBacktrace>();
                Ok(Trap::new(trap, bt.map(Some).unwrap_or(None)))
            }
            None => Err(value),
        }
    }
}

pub fn init() -> Result<(), Error> {
    let class = trap_error();
    class.define_method("message", method!(Trap::message, 0))?;
    class.define_method(
        "wasm_backtrace_message",
        method!(Trap::wasm_backtrace_message, 0),
    )?;
    class.define_method("code", method!(Trap::code, 0))?;
    class.define_method("inspect", method!(Trap::inspect, 0))?;
    class.define_alias("to_s", "message")?;
    Ok(())
}
