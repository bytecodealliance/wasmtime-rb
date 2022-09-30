use crate::{err, error};
use magnus::{Error, Value};
use wasmtime::{Val, ValType};

pub trait ToRubyValue {
    fn to_ruby_value(&self) -> Result<Value, Error>;
}

impl ToRubyValue for Val {
    fn to_ruby_value(&self) -> Result<Value, Error> {
        match self {
            Val::I32(i) => Ok(Value::from(*i)),
            Val::I64(i) => Ok(Value::from(*i)),
            Val::F32(f) => Ok(Value::from(f32::from_bits(*f))),
            Val::F64(f) => Ok(Value::from(f64::from_bits(*f))),
            Val::ExternRef(eref) => match eref {
                None => Ok(magnus::QNIL.into()),
                Some(eref) => eref
                    .data()
                    .downcast_ref::<Value>()
                    .copied()
                    .ok_or_else(|| error!("failed to extract externref")),
            },
            _ => err!("unexpected return type: {:?}", self),
        }
    }
}
pub trait ToWasmVal {
    fn to_wasm_val(&self, ty: ValType) -> Result<Val, Error>;
}

impl ToWasmVal for Value {
    fn to_wasm_val(&self, ty: ValType) -> Result<Val, Error> {
        match ty {
            wasmtime::ValType::I32 => Ok(self.try_convert::<i32>()?.into()),
            wasmtime::ValType::I64 => Ok(self.try_convert::<i64>()?.into()),
            wasmtime::ValType::F32 => Ok(self.try_convert::<f32>()?.into()),
            wasmtime::ValType::F64 => Ok(self.try_convert::<f64>()?.into()),
            _ => err!("unexpected return type: {:?}", ty),
        }
    }
}
