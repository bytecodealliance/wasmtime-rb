use super::convert::ToWasmVal;
use magnus::{exception::arg_error, Error, ExceptionClass, Value};
use wasmtime::ValType;

#[derive(Debug)]
struct Param<'a> {
    index: usize,
    ty: ValType,
    val: &'a Value,
}

impl<'a> Param<'a> {
    pub fn new(index: usize, ty: ValType, val: &'a Value) -> Self {
        Self { index, ty, val }
    }

    fn to_wasmtime_val(&self) -> Result<wasmtime::Val, Error> {
        self.val.to_wasm_val(&self.ty).map_err(|error| match error {
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

pub struct Params<'a>(Vec<ValType>, &'a [Value]);

impl<'a> Params<'a> {
    pub fn new(params_slice: &'a [Value], param_types: Vec<ValType>) -> Result<Self, Error> {
        if param_types.len() != params_slice.len() {
            return Err(Error::new(
                arg_error(),
                format!(
                    "wrong number of arguments (given {}, expected {})",
                    params_slice.len(),
                    param_types.len()
                ),
            ));
        }
        Ok(Self(param_types, params_slice))
    }

    pub fn to_vec(&self) -> Result<Vec<wasmtime::Val>, Error> {
        let mut vals = Vec::with_capacity(self.0.len());
        for (i, (param, value)) in self.0.iter().zip(self.1.iter()).enumerate() {
            let param = Param::new(i, param.clone(), value);
            vals.push(param.to_wasmtime_val()?);
        }

        Ok(vals)
    }
}
