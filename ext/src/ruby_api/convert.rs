use crate::{err, error};
use magnus::{Error, TypedData, Value};
use wasmtime::{ExternRef, Val, ValType};

use super::{func::Func, memory::Memory, store::StoreContextValue};

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
                    .downcast_ref::<OnStackValue>()
                    .map(|v| v.0)
                    .ok_or_else(|| error!("failed to extract externref")),
            },
            _ => err!("unexpected return type: {:?}", self),
        }
    }
}
pub trait ToWasmVal {
    fn to_wasm_val(&self, ty: &ValType) -> Result<Val, Error>;
}

impl ToWasmVal for Value {
    fn to_wasm_val(&self, ty: &ValType) -> Result<Val, Error> {
        match ty {
            ValType::I32 => Ok(self.try_convert::<i32>()?.into()),
            ValType::I64 => Ok(self.try_convert::<i64>()?.into()),
            ValType::F32 => Ok(self.try_convert::<f32>()?.into()),
            ValType::F64 => Ok(self.try_convert::<f64>()?.into()),
            ValType::ExternRef => {
                let extern_ref_value = match self.is_nil() {
                    true => None,
                    false => Some(ExternRef::new(OnStackValue::from(*self))),
                };

                Ok(Val::ExternRef(extern_ref_value))
            }
            _ => err!("unexpected return type: {:?}", ty),
        }
    }
}

struct OnStackValue(Value);
impl From<Value> for OnStackValue {
    fn from(v: Value) -> Self {
        Self(v)
    }
}
unsafe impl Send for OnStackValue {}
unsafe impl Sync for OnStackValue {}

pub trait ToExtern {
    fn to_extern(&self) -> Result<wasmtime::Extern, Error>;
}

impl ToExtern for Value {
    fn to_extern(&self) -> Result<wasmtime::Extern, Error> {
        if self.is_kind_of(Func::class()) {
            Ok(self.try_convert::<&Func>()?.into())
        } else if self.is_kind_of(Memory::class()) {
            Ok(self.try_convert::<&Memory>()?.into())
        } else {
            Err(Error::new(
                magnus::exception::type_error(),
                format!("unexpected extern: {}", self.inspect()),
            ))
        }
    }
}

pub trait WrapWasmtimeType<'a, T>
where
    T: TypedData,
{
    fn wrap_wasmtime_type(&self, store: StoreContextValue<'a>) -> Result<T, Error>;
}
