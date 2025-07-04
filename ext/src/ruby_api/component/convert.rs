use crate::ruby_api::component::component_namespace;
use crate::ruby_api::errors::ExceptionMessage;
use crate::ruby_api::store::StoreContextValue;
use crate::{define_rb_intern, err, error, not_implemented};
use magnus::exception::type_error;
use magnus::rb_sys::AsRawValue;
use magnus::value::{IntoId, Lazy, ReprValue};
use magnus::{
    prelude::*, try_convert, value, Error, IntoValue, RArray, RClass, RHash, RString, Ruby, Value,
};
use wasmtime::component::{Type, Val};

define_rb_intern!(
    // For Component::Result
    OK => "ok",
    ERROR => "error",
    IS_ERROR => "error?",
    IS_OK => "ok?",

    // For Component::Variant
    NEW => "new",
    NAME => "name",
    VALUE => "value",
);

pub(crate) fn component_val_to_rb(val: Val, _store: &StoreContextValue) -> Result<Value, Error> {
    match val {
        Val::Bool(bool) => Ok(bool.into_value()),
        Val::S8(n) => Ok(n.into_value()),
        Val::U8(n) => Ok(n.into_value()),
        Val::S16(n) => Ok(n.into_value()),
        Val::U16(n) => Ok(n.into_value()),
        Val::S32(n) => Ok(n.into_value()),
        Val::U32(n) => Ok(n.into_value()),
        Val::S64(n) => Ok(n.into_value()),
        Val::U64(n) => Ok(n.into_value()),
        Val::Float32(n) => Ok(n.into_value()),
        Val::Float64(n) => Ok(n.into_value()),
        Val::Char(c) => Ok(c.into_value()),
        Val::String(s) => Ok(s.as_str().into_value()),
        Val::List(vec) => {
            let array = RArray::with_capacity(vec.len());
            for val in vec {
                array.push(component_val_to_rb(val, _store)?)?;
            }
            Ok(array.into_value())
        }
        Val::Record(fields) => {
            let hash = RHash::new();
            for (name, val) in fields {
                let rb_value = component_val_to_rb(val, _store)
                    .map_err(|e| e.append(format!(" (struct field \"{name}\")")))?;
                hash.aset(name.as_str(), rb_value)?
            }

            Ok(hash.into_value())
        }
        Val::Tuple(vec) => {
            let array = RArray::with_capacity(vec.len());
            for val in vec {
                array.push(component_val_to_rb(val, _store)?)?;
            }
            Ok(array.into_value())
        }
        Val::Variant(kind, val) => {
            let ruby = Ruby::get().unwrap();
            let payload = match val {
                Some(val) => component_val_to_rb(*val, _store)?,
                None => ruby.qnil().into_value(),
            };

            variant_class(&ruby).funcall(
                NEW.into_id_with(&ruby),
                (kind.into_value_with(&ruby), payload),
            )
        }
        Val::Enum(kind) => Ok(kind.as_str().into_value()),
        Val::Option(val) => match val {
            Some(val) => Ok(component_val_to_rb(*val, _store)?),
            None => Ok(value::qnil().as_value()),
        },
        Val::Result(val) => {
            let ruby = Ruby::get().unwrap();
            let (ruby_method, val) = match val {
                Ok(val) => (OK.into_id_with(&ruby), val),
                Err(val) => (ERROR.into_id_with(&ruby), val),
            };
            let ruby_argument = match val {
                Some(val) => component_val_to_rb(*val, _store)?,
                None => ruby.qnil().as_value(),
            };
            result_class(&ruby).funcall(ruby_method, (ruby_argument,))
        }
        Val::Flags(vec) => Ok(vec.into_value()),
        Val::Resource(_resource_any) => not_implemented!("Resource not implemented"),
    }
}

pub(crate) fn rb_to_component_val(
    value: Value,
    _store: &StoreContextValue,
    ty: &Type,
) -> Result<Val, Error> {
    match ty {
        Type::Bool => {
            let ruby = Ruby::get().unwrap();
            if value.as_raw() == ruby.qtrue().as_raw() {
                Ok(Val::Bool(true))
            } else if value.as_raw() == ruby.qfalse().as_raw() {
                Ok(Val::Bool(false))
            } else {
                Err(Error::new(
                    type_error(),
                    // SAFETY: format will copy classname directly, before we call back in to Ruby
                    format!("no implicit conversion of {} into boolean", unsafe {
                        value.classname()
                    }),
                ))
            }
        }
        Type::S8 => Ok(Val::S8(i8::try_convert(value)?)),
        Type::U8 => Ok(Val::U8(u8::try_convert(value)?)),
        Type::S16 => Ok(Val::S16(i16::try_convert(value)?)),
        Type::U16 => Ok(Val::U16(u16::try_convert(value)?)),
        Type::S32 => Ok(Val::S32(i32::try_convert(value)?)),
        Type::U32 => Ok(Val::U32(u32::try_convert(value)?)),
        Type::S64 => Ok(Val::S64(i64::try_convert(value)?)),
        Type::U64 => Ok(Val::U64(u64::try_convert(value)?)),
        Type::Float32 => Ok(Val::Float32(f32::try_convert(value)?)),
        Type::Float64 => Ok(Val::Float64(f64::try_convert(value)?)),
        Type::Char => Ok(Val::Char(value.to_r_string()?.to_char()?)),
        Type::String => Ok(Val::String(RString::try_convert(value)?.to_string()?)),
        Type::List(list) => {
            let ty = list.ty();
            let rarray = RArray::try_convert(value)?;
            let mut vals: Vec<Val> = Vec::with_capacity(rarray.len());
            // SAFETY: we don't mutate the RArray and we don't call into
            // user code so user code can't mutate it either.
            for (i, value) in unsafe { rarray.as_slice() }.iter().enumerate() {
                let component_val = rb_to_component_val(*value, _store, &ty)
                    .map_err(|e| e.append(format!(" (list item at index {i})")))?;

                vals.push(component_val);
            }
            Ok(Val::List(vals))
        }
        Type::Record(record) => {
            let hash = RHash::try_convert(value)?;

            let mut kv = Vec::with_capacity(record.fields().len());
            for field in record.fields() {
                let value = hash
                    .get(field.name)
                    .ok_or_else(|| error!("struct field missing: {}", field.name))
                    .and_then(|v| {
                        rb_to_component_val(v, _store, &field.ty)
                            .map_err(|e| e.append(format!(" (struct field \"{}\")", field.name)))
                    })?;

                kv.push((field.name.to_string(), value))
            }
            Ok(Val::Record(kv))
        }
        Type::Tuple(tuple) => {
            let types = tuple.types();
            let rarray = RArray::try_convert(value)?;

            if types.len() != rarray.len() {
                return Err(Error::new(
                    magnus::exception::type_error(),
                    format!(
                        "invalid array length for tuple (given {}, expected {})",
                        rarray.len(),
                        types.len()
                    ),
                ));
            }

            let mut vals: Vec<Val> = Vec::with_capacity(rarray.len());

            for (i, (ty, value)) in types.zip(unsafe { rarray.as_slice() }.iter()).enumerate() {
                let component_val = rb_to_component_val(*value, _store, &ty)
                    .map_err(|error| error.append(format!(" (tuple value at index {i})")))?;

                vals.push(component_val);
            }

            Ok(Val::Tuple(vals))
        }
        Type::Variant(variant) => {
            let ruby = Ruby::get().unwrap();

            let name: RString = value.funcall(NAME.into_id_with(&ruby), ())?;
            let name = name.to_string()?;

            let case = variant
                .cases()
                .find(|case| case.name == name.as_str())
                .ok_or_else(|| {
                    error!(
                        "invalid variant case \"{}\", valid cases: {:?}",
                        name,
                        RArray::from_iter(variant.cases().map(|c| c.name))
                    )
                })?;

            let payload_rb: Value = value.funcall(VALUE.into_id_with(&ruby), ())?;
            let payload_val = match (&case.ty, payload_rb.is_nil()) {
                (Some(ty), _) => rb_to_component_val(payload_rb, _store, ty)
                    .map(|val| Some(Box::new(val)))
                    .map_err(|e| e.append(format!(" (variant value for \"{}\")", &name))),

                // case doesn't have payload and Variant#value *is nil*
                (None, true) => Ok(None),

                // case doesn't have payload and Variant#value *is not nil*
                (None, false) => err!(
                    "expected no value for variant case \"{}\", got {}",
                    &name,
                    payload_rb.inspect()
                ),
            }?;

            Ok(Val::Variant(name, payload_val))
        }
        Type::Enum(_) => {
            let rstring = RString::try_convert(value)?;
            rstring.to_string().map(Val::Enum)
        }
        Type::Option(option_type) => {
            if value.is_nil() {
                Ok(Val::Option(None))
            } else {
                Ok(Val::Option(Some(Box::new(rb_to_component_val(
                    value,
                    _store,
                    &option_type.ty(),
                )?))))
            }
        }
        Type::Result(result_type) => {
            // Expect value to conform to `Wasmtime::Component::Value`'s interface
            let ruby = Ruby::get().unwrap();
            let is_ok = value.funcall::<_, (), bool>(IS_OK.into_id_with(&ruby), ())?;

            if is_ok {
                let ok_value = value.funcall::<_, (), Value>(OK.into_id_with(&ruby), ())?;
                match result_type.ok() {
                    Some(ty) => rb_to_component_val(ok_value, _store, &ty)
                        .map(|val| Val::Result(Result::Ok(Some(Box::new(val))))),
                    None => {
                        if ok_value.is_nil() {
                            Ok(Val::Result(Ok(None)))
                        } else {
                            err!(
                                "expected nil for result<_, E> ok branch, got {}",
                                ok_value.inspect()
                            )
                        }
                    }
                }
            } else {
                let err_value = value.funcall::<_, (), Value>(ERROR.into_id_with(&ruby), ())?;
                match result_type.err() {
                    Some(ty) => rb_to_component_val(err_value, _store, &ty)
                        .map(|val| Val::Result(Result::Err(Some(Box::new(val))))),
                    None => {
                        if err_value.is_nil() {
                            Ok(Val::Result(Err(None)))
                        } else {
                            err!(
                                "expected nil for result<O, _> error branch, got {}",
                                err_value.inspect()
                            )
                        }
                    }
                }
            }
        }
        Type::Flags(_) => Vec::<String>::try_convert(value).map(Val::Flags),
        Type::Own(_resource_type) => not_implemented!("Resource not implemented"),
        Type::Borrow(_resource_type) => not_implemented!("Resource not implemented"),
    }
}

fn result_class(ruby: &Ruby) -> RClass {
    static RESULT_CLASS: Lazy<RClass> =
        Lazy::new(|ruby| component_namespace(ruby).const_get("Result").unwrap());
    ruby.get_inner(&RESULT_CLASS)
}

fn variant_class(ruby: &Ruby) -> RClass {
    static VARIANT_CLASS: Lazy<RClass> =
        Lazy::new(|ruby| component_namespace(ruby).const_get("Variant").unwrap());
    ruby.get_inner(&VARIANT_CLASS)
}

pub fn init(ruby: &Ruby) -> Result<(), Error> {
    // Warm up
    let _ = result_class(ruby);
    let _ = OK;
    let _ = ERROR;
    let _ = IS_ERROR;
    let _ = IS_OK;

    let _ = result_class(ruby);
    let _ = NEW;
    let _ = NAME;
    let _ = VALUE;

    Ok(())
}
