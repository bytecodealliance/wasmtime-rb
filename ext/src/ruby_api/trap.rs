use crate::helpers::WrappedStruct;
use crate::ruby_api::{errors::base_error, root};
use magnus::Error;
use magnus::{
    memoize, method, rb_sys::AsRawValue, DataTypeFunctions, ExceptionClass, Module as _, RModule,
    Symbol, TypedData, Value,
};
use wasmtime::TrapCode;

pub fn trap_error() -> ExceptionClass {
    *memoize!(ExceptionClass: root().define_error("Trap", base_error()).unwrap())
}

pub fn trap_code() -> RModule {
    *memoize!(RModule: root().define_module("TrapCode").unwrap())
}

macro_rules! trap_const {
    ($trap:ident) => {
        trap_code().const_get(stringify!($trap)).map(Some)
    };
}

#[derive(TypedData, Debug)]
#[magnus(class = "Wasmtime::Trap", size, free_immediatly)]
/// @yard
pub struct Trap {
    inner: wasmtime::Trap,
}
impl DataTypeFunctions for Trap {}

impl Trap {
    /// @yard
    /// Returns the message with backtrace. Example message:
    ///     wasm trap: wasm `unreachable` instruction executed
    ///     wasm backtrace:
    ///     0:   0x1a - <unknown>!<wasm function 0>
    /// @return [String]
    pub fn message(&self) -> String {
        self.inner.to_string()
    }

    /// @yard
    /// Returns the trap code as a Symbol, possibly nil if the trap did not
    /// origin from Wasm code. All possible trap codes are defined as constants on {Trap}.
    /// @return [Symbol, nil]
    pub fn trap_code(&self) -> Result<Option<Symbol>, Error> {
        if let Some(code) = self.inner.trap_code() {
            match code {
                TrapCode::HeapMisaligned => trap_const!(HEAP_MISALIGNED),
                TrapCode::TableOutOfBounds => trap_const!(TABLE_OUT_OF_BOUNDS),
                TrapCode::IndirectCallToNull => trap_const!(INDIRECT_CALL_TO_NULL),
                TrapCode::BadSignature => trap_const!(BAD_SIGNATURE),
                TrapCode::IntegerOverflow => trap_const!(INTEGER_OVERFLOW),
                TrapCode::IntegerDivisionByZero => trap_const!(INTEGER_DIVISION_BY_ZERO),
                TrapCode::BadConversionToInteger => trap_const!(BAD_CONVERSION_TO_INTEGER),
                TrapCode::UnreachableCodeReached => trap_const!(UNREACHABLE_CODE_REACHED),
                TrapCode::Interrupt => trap_const!(INTERRUPT),
                TrapCode::AlwaysTrapAdapter => trap_const!(ALWAYS_TRAP_ADAPTER),
                // When adding a trap code here, define a matching constant on Wasmtime::Trap (in Ruby)
                _ => trap_const!(UNKNOWN),
            }
        } else {
            Ok(None)
        }
    }

    pub fn inspect(rb_self: WrappedStruct<Self>) -> Result<String, Error> {
        let rs_self = rb_self.get()?;

        Ok(format!(
            "#<Wasmtime::Trap:0x{:016x} @trap_code={}>",
            rb_self.to_value().as_raw(),
            Value::from(rs_self.trap_code()?).inspect()
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

impl From<wasmtime::Trap> for Trap {
    fn from(trap: wasmtime::Trap) -> Self {
        Self { inner: trap }
    }
}

pub fn init() -> Result<(), Error> {
    let class = trap_error();
    class.define_method("message", method!(Trap::message, 0))?;
    class.define_method("trap_code", method!(Trap::trap_code, 0))?;
    class.define_method("inspect", method!(Trap::inspect, 0))?;
    class.define_alias("to_s", "message")?;
    Ok(())
}
