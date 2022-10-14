use super::root;
use crate::{define_rb_intern, err};
use magnus::{function, method, Error, Module as _, Object, RArray, Symbol, Value};
use wasmtime::{FuncType as FuncTypeImpl, ValType};

define_rb_intern!(
    I32 => "i32",
    I64 => "i64",
    F32 => "f32",
    F64 => "f64",
    V128 => "v128",
    FUNCREF => "funcref",
    EXTERNREF => "externref",
);

#[derive(Clone, Debug)]
#[magnus::wrap(class = "Wasmtime::FuncType")]
pub struct FuncType {
    inner: FuncTypeImpl,
}

impl FuncType {
    pub fn get(&self) -> &FuncTypeImpl {
        &self.inner
    }
}

impl FuncType {
    pub fn new(params: RArray, results: RArray) -> Result<Self, Error> {
        let inner = FuncTypeImpl::new(params.to_val_type_vec()?, results.to_val_type_vec()?);
        Ok(Self { inner })
    }

    pub fn params(&self) -> Vec<Symbol> {
        self.get().params().map(ToSym::to_sym).collect()
    }
    pub fn results(&self) -> Vec<Symbol> {
        self.get().results().map(ToSym::to_sym).collect()
    }
}

trait ToValType {
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
            "invalid Webassembly type, expected one of [:i32, :i64, :f32, :f64, :v128, :funcref, :externref], got {:}",
            self.inspect()
        )
    }
}

trait ToValTypeVec {
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

trait ToSym {
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

pub fn init() -> Result<(), Error> {
    let class = root().define_class("FuncType", Default::default())?;

    class.define_singleton_method("new", function!(FuncType::new, 2))?;
    class.define_method("params", method!(FuncType::params, 0))?;
    class.define_method("results", method!(FuncType::results, 0))?;

    Ok(())
}
