use crate::err;
use magnus::{Error, Value};
use wasmtime::{Val, ValType};

#[derive(Debug)]
pub struct ParamTuple<'a>(ValType, &'a Value);

impl<'a> ParamTuple<'a> {
    pub fn new(ty: ValType, val: &'a Value) -> Self {
        Self(ty, val)
    }

    fn to_wasmtime_val(&self) -> Result<wasmtime::Val, Error> {
        match &self.0 {
            ValType::F32 => Ok(Val::F32(self.1.try_convert::<f32>()?.to_bits())),
            ValType::F64 => Ok(Val::F64(self.1.try_convert::<f64>()?.to_bits())),
            ValType::I32 => Ok(Val::I32(self.1.try_convert::<i32>()?)),
            ValType::I64 => Ok(Val::I64(self.1.try_convert::<i64>()?)),
            t => err!("unsupported type {:?}", t),
        }
    }
}

pub struct Params<'a>(Vec<ValType>, &'a [Value]);

impl<'a> Params<'a> {
    pub fn new(params_slice: &'a [Value], param_types: Vec<ValType>) -> Result<Self, Error> {
        if param_types.len() != params_slice.len() {
            return err!(
                "expected {} arguments, got {}",
                param_types.len(),
                params_slice.len()
            );
        }
        Ok(Self(param_types, params_slice))
    }

    pub fn to_vec(&self) -> Result<Vec<wasmtime::Val>, Error> {
        let mut vals = Vec::with_capacity(self.0.len());
        let mut values_iter = self.1.iter();
        for param in &self.0 {
            let tuple = ParamTuple::new(param.clone(), values_iter.next().unwrap());
            vals.push(tuple.to_wasmtime_val()?);
        }
        Ok(vals)
    }
}
