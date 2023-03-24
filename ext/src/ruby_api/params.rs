use super::convert::ToWasmVal;
use magnus::{exception::arg_error, Error, ExceptionClass, Value};
use static_assertions::assert_eq_size;
use wasmtime::{FuncType, ValType};

#[derive(Debug, Copy, Clone)]
#[repr(C)]
struct Param {
    val: Value,
    index: u32,
    ty: ValTypeCopy,
}
// Keep `Param` small so copying it to the stack is cheap, typically anything
// less than 3usize is good
assert_eq_size!(Param, [u64; 2]);

impl Param {
    pub fn new(index: u32, ty: ValType, val: Value) -> Self {
        Self {
            index,
            ty: ty.into(),
            val,
        }
    }

    fn to_wasmtime_val(self) -> Result<wasmtime::Val, Error> {
        self.val
            .to_wasm_val(self.ty.into())
            .map_err(|error| match error {
                Error::Error(class, msg) => {
                    Error::new(class, format!("{} (param index {}) ", msg, self.index))
                }
                Error::Exception(exception) => Error::new(
                    ExceptionClass::from_value(exception.class().into()).unwrap_or_else(arg_error),
                    format!("{} (param index {}) ", exception, self.index),
                ),
                _ => error,
            })
    }
}

pub struct Params<'a>(&'a FuncType, &'a [Value]);

impl<'a> Params<'a> {
    pub fn new(ty: &'a FuncType, params_slice: &'a [Value]) -> Result<Self, Error> {
        if ty.params().len() != params_slice.len() {
            return Err(Error::new(
                arg_error(),
                format!(
                    "wrong number of arguments (given {}, expected {})",
                    params_slice.len(),
                    ty.params().len()
                ),
            ));
        }
        Ok(Self(ty, params_slice))
    }

    pub fn to_vec(&self) -> Result<Vec<wasmtime::Val>, Error> {
        let mut vals = Vec::with_capacity(self.0.params().len());
        for (i, (param, value)) in self.0.params().zip(self.1.iter()).enumerate() {
            let i: u32 = i
                .try_into()
                .map_err(|_| Error::new(arg_error(), "too many params"))?;
            let param = Param::new(i, param.clone(), *value);
            vals.push(param.to_wasmtime_val()?);
        }

        Ok(vals)
    }
}

/// A [`wasmtime::ValType`] that is [`Copy`], so it can be stays on the stack
#[derive(Debug, Clone, Copy)]
pub enum ValTypeCopy {
    I32,
    I64,
    F32,
    F64,
    V128,
    FuncRef,
    ExternRef,
}

impl From<ValType> for ValTypeCopy {
    fn from(ty: ValType) -> Self {
        match ty {
            ValType::I32 => Self::I32,
            ValType::I64 => Self::I64,
            ValType::F32 => Self::F32,
            ValType::F64 => Self::F64,
            ValType::V128 => Self::V128,
            ValType::FuncRef => Self::FuncRef,
            ValType::ExternRef => Self::ExternRef,
        }
    }
}

impl From<ValTypeCopy> for ValType {
    fn from(ty: ValTypeCopy) -> Self {
        match ty {
            ValTypeCopy::I32 => Self::I32,
            ValTypeCopy::I64 => Self::I64,
            ValTypeCopy::F32 => Self::F32,
            ValTypeCopy::F64 => Self::F64,
            ValTypeCopy::V128 => Self::V128,
            ValTypeCopy::FuncRef => Self::FuncRef,
            ValTypeCopy::ExternRef => Self::ExternRef,
        }
    }
}
