use crate::ruby_api::{
    component::{
        convert::{component_val_to_rb, rb_to_component_val},
        Instance,
    },
    errors::ExceptionMessage,
    store::{Store, StoreContextValue},
};
use magnus::{
    class, exception::arg_error, gc::Marker, method, prelude::*, typed_data::Obj, value,
    DataTypeFunctions, Error, IntoValue, RArray, RModule, Ruby, TypedData, Value,
};
use wasmtime::component::{Func as FuncImpl, Type, Val};

/// @yard
/// @rename Wasmtime::Component::Func
/// Represents a WebAssembly component Function
/// @see https://docs.wasmtime.dev/api/wasmtime/component/struct.Func.html Wasmtime's Rust doc
///
/// == Component model types conversion
///
/// Here's how component model types map to Ruby objects:
///
/// bool::
///     Ruby +true+ or +false+, no automatic conversion happens.
/// s8, u8, s16, u16, etc.::
///     Ruby +Integer+. Overflows raise.
/// f32, f64::
///     Ruby +Float+.
/// string::
///     Ruby +String+. Exception will be raised if the string is not valid UTF-8.
/// list<T>::
///     Ruby +Array+.
/// tuple::
///     Ruby +Array+ of the same size of tuple. Example: +tuple<T, U>+ would be converted to +[T, U]+.
/// record::
///     Ruby +Hash+ where field names are +String+s.
/// result<O, E>::
///     {Result} instance. When converting a result branch of the none
///     type, the {Result}’s value MUST be +nil+.
///
///     Examples of none type in a result: unparametrized +result+, +result<O>+, +result<_, E>+.
/// option<T>::
///     +nil+ is mapped to +None+, anything else is mapped to +Some(T)+.
/// flags::
///     Ruby +Array+ of +String+s.
/// enum::
///     Ruby +String+. Exception will be raised of the +String+ is not a valid enum value.
/// variant::
///     {Variant} instance wrapping the variant's name and optionally its value.
///     Exception will be raised for:
///     - invalid {Variant#name},
///     - unparametrized variant and not nil {Variant#value}.
/// resource (own<T> or borrow<T>)::
///     Not yet supported.
#[derive(TypedData)]
#[magnus(class = "Wasmtime::Component::Func", size, mark, free_immediately)]
pub struct Func {
    store: Obj<Store>,
    instance: Obj<Instance>,
    inner: FuncImpl,
}
unsafe impl Send for Func {}

impl DataTypeFunctions for Func {
    fn mark(&self, marker: &Marker) {
        marker.mark(self.store);
        marker.mark(self.instance);
    }
}

impl Func {
    /// @yard
    /// Calls a Wasm component model function.
    /// @def call(*args)
    /// @param args [Array<Object>] the function's arguments as per its Wasm definition
    /// @return [Object] the function's return value as per its Wasm definition
    /// @see Func Func class-level documentation for type conversion logic
    pub fn call(&self, args: &[Value]) -> Result<Value, Error> {
        Func::invoke(self.store, &self.inner, args)
    }

    pub fn from_inner(inner: FuncImpl, instance: Obj<Instance>, store: Obj<Store>) -> Self {
        Self {
            store,
            instance,
            inner,
        }
    }

    pub fn invoke(store: Obj<Store>, func: &FuncImpl, args: &[Value]) -> Result<Value, Error> {
        let store_context_value = StoreContextValue::from(store);
        let results_ty = func.results(store.context_mut());
        let mut results = vec![wasmtime::component::Val::Bool(false); results_ty.len()];
        let params = convert_params(
            &store_context_value,
            &func.params(store.context_mut()),
            args,
        )?;

        func.call(store.context_mut(), &params, &mut results)
            .map_err(|e| store_context_value.handle_wasm_error(e))?;

        let result = match results_ty.len() {
            0 => Ok(value::qnil().as_value()),
            1 => component_val_to_rb(results.into_iter().next().unwrap(), &store_context_value),
            _ => results
                .into_iter()
                .map(|val| component_val_to_rb(val, &store_context_value))
                .collect::<Result<RArray, Error>>()
                .map(IntoValue::into_value),
        };

        func.post_return(store.context_mut())
            .map_err(|e| store_context_value.handle_wasm_error(e))?;

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

        let component_val = rb_to_component_val(*value, store, ty)
            .map_err(|error| error.append(format!(" (param at index {})", i)))?;

        params.push(component_val);
    }

    Ok(params)
}

pub fn init(_ruby: &Ruby, namespace: &RModule) -> Result<(), Error> {
    let func = namespace.define_class("Func", class::object())?;
    func.define_method("call", method!(Func::call, -1))?;

    Ok(())
}
