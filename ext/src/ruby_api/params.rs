use super::convert::ToWasmVal;
use magnus::{exception::arg_error, Error, ExceptionClass, Value};
use static_assertions::assert_eq_size;
use wasmtime::{FuncType, ValType};

#[derive(Debug, Copy, Clone)]
#[repr(C)]
struct Param {
    val: Value,
    index: u32,
    ty: ValType,
}
// Keep `Param` small so copying it to the stack is cheap, typically anything
// less than 3usize is good
assert_eq_size!(Param, [u64; 2]);

impl Param {
    pub fn new(index: u32, ty: ValType, val: Value) -> Self {
        Self { index, ty, val }
    }

    fn to_wasmtime_val(self) -> Result<wasmtime::Val, Error> {
        self.val.to_wasm_val(self.ty).map_err(|error| match error {
            Error::Error(class, msg) => {
                Error::new(class, format!("{} (param at index {})", msg, self.index))
            }
            Error::Exception(exception) => Error::new(
                ExceptionClass::from_value(exception.class().into()).unwrap_or_else(arg_error),
                format!("{} (param at index {})", exception, self.index),
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
            let param = Param::new(i, param, *value);
            vals.push(param.to_wasmtime_val()?);
        }

        Ok(vals)
    }
}
