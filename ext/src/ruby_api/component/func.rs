use super::convert::ToComponentVal;
use crate::{
    err, error,
    ruby_api::{convert::ToRubyValue, errors::ExceptionMessage, store::StoreContextValue},
};
use magnus::{
    class,
    error::ErrorType,
    exception::arg_error,
    typed_data::Obj,
    value::{self, ReprValue},
    DataTypeFunctions, Error, Module as _, Object, RArray, Ruby, TryConvert, TypedData, Value,
};
use magnus::{IntoValue, RModule};
use std::{borrow::BorrowMut, cell::RefCell};
use wasmtime::component::{Func as FuncImpl, Type, Val};

pub struct Func;

impl Func {
    pub fn invoke(
        store: &StoreContextValue,
        func: &FuncImpl,
        args: &[Value],
    ) -> Result<Value, Error> {
        let results_ty = func.results(store.context()?);
        let mut results = vec![wasmtime::component::Val::Bool(false); results_ty.len()];
        let params = convert_params(store, &func.params(store.context()?), args)?;

        func.call(store.context_mut()?, &params, &mut results)
            .map_err(|e| error!("{}", e))?;

        let result = match results_ty.len() {
            0 => Ok(value::qnil().as_value()),
            1 => Ok(results.first().unwrap().to_ruby_value(store)?),
            _ => Ok(results
                .iter()
                .map(|v| v.to_ruby_value(store))
                .collect::<Result<RArray, Error>>()?
                .into_value()),
        };

        func.post_return(store.context_mut()?)
            .map_err(|e| error!("{}", e))?; // TODO: should this be a Wasmtime::Error::Trap?

        result
    }
}

fn convert_params(
    store: &StoreContextValue,
    ty: &[Type],
    params_slice: &[Value],
) -> Result<Vec<Val>, Error> {
    if ty.len() != params_slice.len() {
        return Err(Error::new(
            arg_error(),
            format!(
                "wrong number of arguments (given {}, expected {})",
                params_slice.len(),
                ty.len()
            ),
        ));
    }

    let mut params = Vec::with_capacity(ty.len());
    for (i, (ty, value)) in ty.iter().zip(params_slice.iter()).enumerate() {
        let i: u32 = i
            .try_into()
            .map_err(|_| Error::new(arg_error(), "too many params"))?;

        let component_val = value
            .to_component_val(store, ty)
            .map_err(|error| error.append(format!(" (param at index {})", i)))?;

        params.push(component_val);
    }

    Ok(params)
}
