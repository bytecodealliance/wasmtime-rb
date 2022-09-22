use crate::err;
use magnus::{Error, Fixnum, RFloat, Value};
use wasmtime::Val;

pub trait ToRubyValue {
    fn to_ruby_value(&self) -> Result<Value, Error>;
}

impl ToRubyValue for Val {
    fn to_ruby_value(&self) -> Result<Value, Error> {
        match *self {
            Val::I32(i) => Ok(*Fixnum::from_i64(i.into()).unwrap()),
            Val::I64(i) => Ok(*Fixnum::from_i64(i).unwrap()),
            Val::F32(f) => Ok(*RFloat::from_f64(f32::from_bits(f).into()).unwrap()),
            Val::F64(f) => Ok(*RFloat::from_f64(f64::from_bits(f)).unwrap()),
            _ => err!("unexpected return type: {:?}", self),
        }
    }
}
