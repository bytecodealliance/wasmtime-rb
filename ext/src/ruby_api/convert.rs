use crate::{define_rb_intern, err, error, helpers::SymbolEnum};
use lazy_static::lazy_static;
use magnus::{prelude::*, Error, IntoValue, RArray, Ruby, Symbol, TryConvert, TypedData, Value};
use wasmtime::{ExternRef, RefType, Val, ValType};

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
            (*FUNCREF, ValType::FUNCREF),
            (*EXTERNREF, ValType::EXTERNREF),
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
                    .data(store.context()?)
                    .map_err(|e| error!("{e}"))?
                    .downcast_ref::<ExternRefValue>()
                    .map(|v| v.0)
                    .ok_or_else(|| error!("failed to extract externref")),
            },
            Val::FuncRef(funcref) => match funcref {
                None => Ok(().into_value()),
                Some(funcref) => Ok(Func::from_inner(*store, *funcref).into_value()),
            },
            Val::V128(_) => err!("converting from v128 to Ruby unsupported"),
            t => err!("cannot convert value: {t:?} to Ruby value"),
        }
    }
}
pub trait ToWasmVal {
    fn to_wasm_val(&self, store: &StoreContextValue, ty: ValType) -> Result<Val, Error>;
}

impl ToWasmVal for Value {
    fn to_wasm_val(&self, store: &StoreContextValue, ty: ValType) -> Result<Val, Error> {
        if ty.matches(&ValType::EXTERNREF) {
            // Don't special case `nil` in order to ensure that it's always
            // a rooted value. Even though it's `nil` from Ruby's perspective,
            // it's a host managed object.
            let extern_ref_value = Some(
                ExternRef::new(
                    store.context_mut().map_err(|e| error!("{e}"))?,
                    ExternRefValue::from(*self),
                )
                .map_err(|e| error!("{e}"))?,
            );

            return Ok(Val::ExternRef(extern_ref_value));
        }

        if ty.matches(&ValType::FUNCREF) {
            let func_ref_value = match self.is_nil() {
                true => None,
                false => Some(*<&Func>::try_convert(*self)?.inner()),
            };
            return Ok(Val::FuncRef(func_ref_value));
        }

        match ty {
            ValType::I32 => Ok(i32::try_convert(*self)?.into()),
            ValType::I64 => Ok(i64::try_convert(*self)?.into()),
            ValType::F32 => Ok(f32::try_convert(*self)?.into()),
            ValType::F64 => Ok(f64::try_convert(*self)?.into()),
            ValType::V128 => err!("converting from Ruby to v128 not supported"),
            // TODO: to be filled in once typed function references and/or GC
            // are enabled by default.
            t => err!("unsupported type: {t:?}"),
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
    fn to_extern(&self, ruby: &Ruby) -> Result<wasmtime::Extern, Error>;
}

impl ToExtern for Value {
    fn to_extern(&self, ruby: &Ruby) -> Result<wasmtime::Extern, Error> {
        if self.is_kind_of(Func::class(ruby)) {
            Ok(<&Func>::try_convert(*self)?.into())
        } else if self.is_kind_of(Memory::class(ruby)) {
            Ok(<&Memory>::try_convert(*self)?.into())
        } else if self.is_kind_of(Table::class(ruby)) {
            Ok(<&Table>::try_convert(*self)?.into())
        } else if self.is_kind_of(Global::class(ruby)) {
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
    fn to_sym(&self) -> Result<Symbol, Error>;
}

impl ToSym for ValType {
    fn to_sym(&self) -> Result<Symbol, Error> {
        if self.matches(&ValType::EXTERNREF) {
            return Ok(Symbol::from(*EXTERNREF));
        }

        if self.matches(&ValType::FUNCREF) {
            return Ok(Symbol::from(*FUNCREF));
        }

        match self {
            &ValType::I32 => Ok(Symbol::from(*I32)),
            &ValType::I64 => Ok(Symbol::from(*I64)),
            &ValType::F32 => Ok(Symbol::from(*F32)),
            &ValType::F64 => Ok(Symbol::from(*F64)),
            &ValType::V128 => Ok(Symbol::from(*V128)),
            t => Err(error!("Unsupported type {t:?}")),
        }
    }
}

impl ToSym for RefType {
    fn to_sym(&self) -> Result<Symbol, Error> {
        if self.matches(&RefType::FUNCREF) {
            return Ok(Symbol::from(*FUNCREF));
        }
        if self.matches(&RefType::EXTERNREF) {
            return Ok(Symbol::from(*EXTERNREF));
        }
        Err(error!("Unsupported RefType {self:}"))
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
