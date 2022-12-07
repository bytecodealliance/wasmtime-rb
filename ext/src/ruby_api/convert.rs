use crate::{define_rb_intern, err, error};
use magnus::{Error, Symbol, TypedData, Value};
use wasmtime::{ExternRef, Val, ValType};

use super::{func::Func, memory::Memory, store::StoreContextValue};

define_rb_intern!(
    I32 => "i32",
    I64 => "i64",
    F32 => "f32",
    F64 => "f64",
    V128 => "v128",
    FUNCREF => "funcref",
    EXTERNREF => "externref",
);

pub trait ToRubyValue {
    fn to_ruby_value(&self, store: &StoreContextValue) -> Result<Value, Error>;
}

impl ToRubyValue for Val {
    fn to_ruby_value(&self, store: &StoreContextValue) -> Result<Value, Error> {
        match self {
            Val::I32(i) => Ok(Value::from(*i)),
            Val::I64(i) => Ok(Value::from(*i)),
            Val::F32(f) => Ok(Value::from(f32::from_bits(*f))),
            Val::F64(f) => Ok(Value::from(f64::from_bits(*f))),
            Val::ExternRef(eref) => match eref {
                None => Ok(magnus::QNIL.into()),
                Some(eref) => eref
                    .data()
                    .downcast_ref::<ExternRefValue>()
                    .map(|v| v.0)
                    .ok_or_else(|| error!("failed to extract externref")),
            },
            Val::FuncRef(funcref) => match funcref {
                None => Ok(magnus::QNIL.into()),
                Some(funcref) => Ok(Func::from_inner(*store, *funcref).into()),
            },
            Val::V128(_) => err!("converting from v128 to Ruby unsupported"),
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
                    false => Some(ExternRef::new(ExternRefValue::from(*self))),
                };

                Ok(Val::ExternRef(extern_ref_value))
            }
            ValType::FuncRef => {
                let func_ref_value = match self.is_nil() {
                    true => None,
                    false => Some(*self.try_convert::<&Func>()?.inner()),
                };
                Ok(Val::FuncRef(func_ref_value))
            }
            ValType::V128 => err!("converting from Ruby to v128 not supported"),
        }
    }
}

struct ExternRefValue(Value);
impl From<Value> for ExternRefValue {
    fn from(v: Value) -> Self {
        Self(v)
    }
}
unsafe impl Send for ExternRefValue {}
unsafe impl Sync for ExternRefValue {}

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

pub trait ToSym {
    fn to_sym(self) -> Symbol;
}

impl ToSym for ValType {
    fn to_sym(self) -> Symbol {
        match self {
            ValType::I32 => Symbol::from(*I32),
            ValType::I64 => Symbol::from(*I64),
            ValType::F32 => Symbol::from(*F32),
            ValType::F64 => Symbol::from(*F64),
            ValType::V128 => Symbol::from(*V128),
            ValType::FuncRef => Symbol::from(*FUNCREF),
            ValType::ExternRef => Symbol::from(*EXTERNREF),
        }
    }
}
pub trait ToValType {
    fn to_val_type(&self) -> Result<ValType, Error>;
}

impl ToValType for Value {
    fn to_val_type(&self) -> Result<ValType, Error> {
        if let Ok(symbol) = self.try_convert::<Symbol>() {
            if let Ok(true) = symbol.equal(Symbol::from(*I32)) {
                return Ok(ValType::I32);
            }
            if let Ok(true) = symbol.equal(Symbol::from(*I64)) {
                return Ok(ValType::I64);
            }
            if let Ok(true) = symbol.equal(Symbol::from(*F32)) {
                return Ok(ValType::F32);
            }
            if let Ok(true) = symbol.equal(Symbol::from(*F64)) {
                return Ok(ValType::F64);
            }
            if let Ok(true) = symbol.equal(Symbol::from(*V128)) {
                return Ok(ValType::V128);
            }
            if let Ok(true) = symbol.equal(Symbol::from(*FUNCREF)) {
                return Ok(ValType::FuncRef);
            }
            if let Ok(true) = symbol.equal(Symbol::from(*EXTERNREF)) {
                return Ok(ValType::ExternRef);
            }
        }

        err!(
            "invalid WebAssembly type, expected one of [:i32, :i64, :f32, :f64, :v128, :funcref, :externref], got {:}",
            self.inspect()
        )
    }
}

pub trait WrapWasmtimeType<'a, T>
where
    T: TypedData,
{
    fn wrap_wasmtime_type(&self, store: StoreContextValue<'a>) -> Result<T, Error>;
}
