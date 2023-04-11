use crate::{define_rb_intern, err, error, helpers::SymbolEnum};
use lazy_static::lazy_static;
use magnus::{Error, IntoValue, RArray, Symbol, TryConvert, TypedData, Value};
use wasmtime::{ExternRef, Val, ValType};

use super::{func::Func, global::Global, memory::Memory, store::StoreContextValue, table::Table};

define_rb_intern!(
    I32 => "i32",
    I64 => "i64",
    F32 => "f32",
    F64 => "f64",
    V128 => "v128",
    FUNCREF => "funcref",
    EXTERNREF => "externref",
);

lazy_static! {
    static ref VALTYPE_MAPPING: SymbolEnum<'static, ValType> = {
        let mapping = vec![
            (*I32, ValType::I32),
            (*I64, ValType::I64),
            (*F32, ValType::F32),
            (*F64, ValType::F64),
            (*V128, ValType::V128),
            (*FUNCREF, ValType::FuncRef),
            (*EXTERNREF, ValType::ExternRef),
        ];

        SymbolEnum::new("WebAssembly type", mapping)
    };
}

pub trait ToRubyValue {
    fn to_ruby_value(&self, store: &StoreContextValue) -> Result<Value, Error>;
}

impl ToRubyValue for Val {
    fn to_ruby_value(&self, store: &StoreContextValue) -> Result<Value, Error> {
        match self {
            Val::I32(i) => Ok(i.into_value()),
            Val::I64(i) => Ok(i.into_value()),
            Val::F32(f) => Ok(f32::from_bits(*f).into_value()),
            Val::F64(f) => Ok(f64::from_bits(*f).into_value()),
            Val::ExternRef(eref) => match eref {
                None => Ok(().into_value()),
                Some(eref) => eref
                    .data()
                    .downcast_ref::<ExternRefValue>()
                    .map(|v| v.0)
                    .ok_or_else(|| error!("failed to extract externref")),
            },
            Val::FuncRef(funcref) => match funcref {
                None => Ok(().into_value()),
                Some(funcref) => Ok(Func::from_inner(*store, *funcref).into_value()),
            },
            Val::V128(_) => err!("converting from v128 to Ruby unsupported"),
        }
    }
}
pub trait ToWasmVal {
    fn to_wasm_val(&self, ty: ValType) -> Result<Val, Error>;
}

impl ToWasmVal for Value {
    fn to_wasm_val(&self, ty: ValType) -> Result<Val, Error> {
        match ty {
            ValType::I32 => Ok(i32::try_convert(*self)?.into()),
            ValType::I64 => Ok(i64::try_convert(*self)?.into()),
            ValType::F32 => Ok(f32::try_convert(*self)?.into()),
            ValType::F64 => Ok(f64::try_convert(*self)?.into()),
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
                    false => Some(*<&Func>::try_convert(*self)?.inner()),
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
            Ok(<&Func>::try_convert(*self)?.into())
        } else if self.is_kind_of(Memory::class()) {
            Ok(<&Memory>::try_convert(*self)?.into())
        } else if self.is_kind_of(Table::class()) {
            Ok(<&Table>::try_convert(*self)?.into())
        } else if self.is_kind_of(Global::class()) {
            Ok(<&Global>::try_convert(*self)?.into())
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
        VALTYPE_MAPPING.get(*self)
    }
}

impl ToValType for Symbol {
    fn to_val_type(&self) -> Result<ValType, Error> {
        self.as_value().to_val_type()
    }
}

pub trait ToValTypeVec {
    fn to_val_type_vec(&self) -> Result<Vec<ValType>, Error>;
}

impl ToValTypeVec for RArray {
    fn to_val_type_vec(&self) -> Result<Vec<ValType>, Error> {
        unsafe { self.as_slice() }
            .iter()
            .map(ToValType::to_val_type)
            .collect::<Result<Vec<ValType>, Error>>()
    }
}

pub trait WrapWasmtimeType<'a, T>
where
    T: TypedData,
{
    fn wrap_wasmtime_type(&self, store: StoreContextValue<'a>) -> Result<T, Error>;
}
