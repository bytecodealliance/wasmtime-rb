use super::{convert::ToWasmVal, errors::ExceptionMessage, store::StoreContextValue};
use magnus::{error::ErrorType, exception::arg_error, Error, Value};
use static_assertions::assert_eq_size;
use wasmtime::{FuncType, ValType};

#[derive(Debug, Clone)]
#[repr(C)]
struct Param {
    val: Value,
    index: u32,
    ty: ValType,
}

impl Param {
    pub fn new(index: u32, ty: ValType, val: Value) -> Self {
        Self { index, ty, val }
    }

    fn to_wasmtime_val(&self, store: &StoreContextValue) -> Result<wasmtime::Val, Error> {
        self.val
            .to_wasm_val(store, self.ty.clone())
            .map_err(|error| error.append(format!(" (param at index {})", self.index)))
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

    pub fn to_vec(&self, store: &StoreContextValue) -> Result<Vec<wasmtime::Val>, Error> {
        let mut vals = Vec::with_capacity(self.0.params().len());
        for (i, (param, value)) in self.0.params().zip(self.1.iter()).enumerate() {
            let i: u32 = i
                .try_into()
                .map_err(|_| Error::new(arg_error(), "too many params"))?;
            let param = Param::new(i, param, *value);
            vals.push(param.to_wasmtime_val(store)?);
        }

        Ok(vals)
    }
}
