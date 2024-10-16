use crate::not_implemented;
use crate::ruby_api::errors::ExceptionMessage;
use crate::ruby_api::{convert::ToRubyValue, store::StoreContextValue};
use crate::{err, error, helpers::SymbolEnum};
use magnus::exception::type_error;
use magnus::rb_sys::AsRawValue;
use magnus::value::{qtrue, ReprValue};
use magnus::{
    prelude::*, value, Error, IntoValue, RArray, RHash, RString, Ruby, Symbol, TryConvert,
    TypedData, Value,
};
use wasmtime::component::{Type, Val};

impl ToRubyValue for Val {
    fn to_ruby_value(&self, _store: &StoreContextValue) -> Result<Value, Error> {
        match self {
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
                    array.push(val.to_ruby_value(_store)?)?;
                }
                Ok(array.into_value())
            }
            Val::Record(fields) => {
                let hash = RHash::new();
                for (name, component_val) in fields {
                    let ruby_value = component_val
                        .to_ruby_value(_store)
                        .map_err(|e| e.append(format!(" (struct field \"{}\")", name)))?;
                    hash.aset(name.as_str(), ruby_value)?
                }

                Ok(hash.into_value())
            }
            Val::Tuple(vec) => {
                let array = RArray::with_capacity(vec.len());
                for val in vec {
                    array.push(val.to_ruby_value(_store)?)?;
                }
                Ok(array.into_value())
            }
            Val::Variant(_kind, _val) => not_implemented!("Variant not implemented"),
            Val::Enum(kind) => Ok(kind.as_str().into_value()),
            Val::Option(val) => match val {
                Some(val) => Ok(val.to_ruby_value(_store)?),
                None => Ok(value::qnil().as_value()),
            },
            Val::Result(_val) => not_implemented!("Result not implemented"),
            Val::Flags(_vec) => not_implemented!("Flags not implemented"),
            Val::Resource(_resource_any) => not_implemented!("Resource not implemented"),
        }
    }
}

pub trait ToComponentVal {
    fn to_component_val(&self, store: &StoreContextValue, ty: &Type) -> Result<Val, Error>;
}

impl ToComponentVal for Value {
    fn to_component_val(&self, _store: &StoreContextValue, ty: &Type) -> Result<Val, Error> {
        match ty {
            Type::Bool => {
                let ruby = Ruby::get().unwrap();
                if self.as_raw() == ruby.qtrue().as_raw() {
                    Ok(Val::Bool(true))
                } else if self.as_raw() == ruby.qfalse().as_raw() {
                    Ok(Val::Bool(false))
                } else {
                    Err(Error::new(
                        type_error(),
                        // SAFETY: format will copy classname directly, before we call back in to Ruby
                        format!("no implicit conversion of {} into boolean", unsafe {
                            self.classname()
                        }),
                    ))
                }
            }
            Type::S8 => Ok(Val::S8(i8::try_convert(*self)?)),
            Type::U8 => Ok(Val::U8(u8::try_convert(*self)?)),
            Type::S16 => Ok(Val::S16(i16::try_convert(*self)?)),
            Type::U16 => Ok(Val::U16(u16::try_convert(*self)?)),
            Type::S32 => Ok(Val::S32(i32::try_convert(*self)?)),
            Type::U32 => Ok(Val::U32(u32::try_convert(*self)?)),
            Type::S64 => Ok(Val::S64(i64::try_convert(*self)?)),
            Type::U64 => Ok(Val::U64(u64::try_convert(*self)?)),
            Type::Float32 => Ok(Val::Float32(f32::try_convert(*self)?)),
            Type::Float64 => Ok(Val::Float64(f64::try_convert(*self)?)),
            Type::Char => Ok(Val::Char(self.to_r_string()?.to_char()?)),
            Type::String => Ok(Val::String(RString::try_convert(*self)?.to_string()?)),
            Type::List(list) => {
                let ty = list.ty();
                let rarray = RArray::try_convert(*self)?;
                let mut vals: Vec<Val> = Vec::with_capacity(rarray.len());
                // SAFETY: we don't mutate the RArray and we don't call into
                // user code so user code can't mutate it either.
                for (i, value) in unsafe { rarray.as_slice() }.iter().enumerate() {
                    let component_val = value
                        .to_component_val(_store, &ty)
                        .map_err(|e| e.append(format!(" (list item at index {})", i)))?;

                    vals.push(component_val);
                }
                Ok(Val::List(vals))
            }
            Type::Record(record) => {
                let hash = RHash::try_convert(*self)
                    .map_err(|_| error!("Invalid value for record: {}", self.inspect()))?;

                let mut kv = Vec::with_capacity(record.fields().len());
                for field in record.fields() {
                    let value = hash
                        .aref::<_, Value>(field.name)
                        .map_err(|_| error!("Struct field missing: {}", field.name))?
                        .to_component_val(_store, &field.ty)
                        .map_err(|e| e.append(format!(" (struct field \"{}\")", field.name)))?;

                    kv.push((field.name.to_string(), value))
                }
                Ok(Val::Record(kv))
            }
            Type::Tuple(tuple) => {
                let types = tuple.types();
                let rarray = RArray::try_convert(*self)?;

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
                    let component_val = value
                        .to_component_val(_store, &ty)
                        .map_err(|error| error.append(format!(" (tuple value at index {})", i)))?;

                    vals.push(component_val);
                }

                Ok(Val::Tuple(vals))
            }
            Type::Variant(_variant) => not_implemented!("Variant not implemented"),
            Type::Enum(_enum) => not_implemented!("Enum not implementend"),
            Type::Option(option_type) => {
                if self.is_nil() {
                    Ok(Val::Option(None))
                } else {
                    Ok(Val::Option(Some(Box::new(
                        self.to_component_val(_store, &option_type.ty())?,
                    ))))
                }
            }
            Type::Result(_result_type) => not_implemented!("Result not implemented"),
            Type::Flags(_flags) => not_implemented!("Flags not implemented"),
            Type::Own(_resource_type) => not_implemented!("Resource not implemented"),
            Type::Borrow(_resource_type) => not_implemented!("Resource not implemented"),
        }
    }
}
