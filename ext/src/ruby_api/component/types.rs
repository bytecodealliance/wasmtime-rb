use crate::error;
use magnus::{
    class, function, gc::Marker, method, prelude::*, r_hash::ForEach, value::Opaque, Error,
    Module as _, RArray, RHash, RString, Ruby, Symbol, TryConvert, TypedData, Value,
};
use std::fmt;

/// Standalone component type system that can be constructed independently
/// of a component instance. Used for defining host function signatures.
#[derive(Clone, Debug)]
pub enum ComponentType {
    Bool,
    S8,
    U8,
    S16,
    U16,
    S32,
    U32,
    S64,
    U64,
    Float32,
    Float64,
    Char,
    String,
    List(Box<ComponentType>),
    Record(Vec<RecordField>),
    Tuple(Vec<ComponentType>),
    Variant(Vec<VariantCase>),
    Enum(Vec<String>),
    Option(Box<ComponentType>),
    Result {
        ok: Option<Box<ComponentType>>,
        err: Option<Box<ComponentType>>,
    },
    Flags(Vec<String>),
}

#[derive(Clone, Debug)]
pub struct RecordField {
    pub name: String,
    pub ty: ComponentType,
}

#[derive(Clone, Debug)]
pub struct VariantCase {
    pub name: String,
    pub ty: Option<ComponentType>,
}

impl fmt::Display for ComponentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComponentType::Bool => write!(f, "bool"),
            ComponentType::S8 => write!(f, "s8"),
            ComponentType::U8 => write!(f, "u8"),
            ComponentType::S16 => write!(f, "s16"),
            ComponentType::U16 => write!(f, "u16"),
            ComponentType::S32 => write!(f, "s32"),
            ComponentType::U32 => write!(f, "u32"),
            ComponentType::S64 => write!(f, "s64"),
            ComponentType::U64 => write!(f, "u64"),
            ComponentType::Float32 => write!(f, "float32"),
            ComponentType::Float64 => write!(f, "float64"),
            ComponentType::Char => write!(f, "char"),
            ComponentType::String => write!(f, "string"),
            ComponentType::List(inner) => write!(f, "list<{}>", inner),
            ComponentType::Record(fields) => {
                write!(f, "record {{")?;
                for (i, field) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", field.name, field.ty)?;
                }
                write!(f, "}}")
            }
            ComponentType::Tuple(types) => {
                write!(f, "tuple<")?;
                for (i, ty) in types.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", ty)?;
                }
                write!(f, ">")
            }
            ComponentType::Variant(cases) => {
                write!(f, "variant {{")?;
                for (i, case) in cases.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", case.name)?;
                    if let Some(ty) = &case.ty {
                        write!(f, "({})", ty)?;
                    }
                }
                write!(f, "}}")
            }
            ComponentType::Enum(cases) => {
                write!(f, "enum {{")?;
                for (i, case) in cases.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", case)?;
                }
                write!(f, "}}")
            }
            ComponentType::Option(inner) => write!(f, "option<{}>", inner),
            ComponentType::Result { ok, err } => {
                write!(f, "result<")?;
                if let Some(ok) = ok {
                    write!(f, "{}", ok)?;
                } else {
                    write!(f, "_")?;
                }
                write!(f, ", ")?;
                if let Some(err) = err {
                    write!(f, "{}", err)?;
                } else {
                    write!(f, "_")?;
                }
                write!(f, ">")
            }
            ComponentType::Flags(flags) => {
                write!(f, "flags {{")?;
                for (i, flag) in flags.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", flag)?;
                }
                write!(f, "}}")
            }
        }
    }
}

/// @yard
/// @rename Wasmtime::Component::Type
/// Ruby wrapper for ComponentType - stored as opaque Rust data
/// Factory methods for creating component types
/// @see https://docs.wasmtime.dev/api/wasmtime/component/enum.Val.html
///
/// @!method self.bool
///   @return [Type] A boolean type
/// @!method self.s8
///   @return [Type] A signed 8-bit integer type
/// @!method self.u8
///   @return [Type] An unsigned 8-bit integer type
/// @!method self.s16
///   @return [Type] A signed 16-bit integer type
/// @!method self.u16
///   @return [Type] An unsigned 16-bit integer type
/// @!method self.s32
///   @return [Type] A signed 32-bit integer type
/// @!method self.u32
///   @return [Type] An unsigned 32-bit integer type
/// @!method self.s64
///   @return [Type] A signed 64-bit integer type
/// @!method self.u64
///   @return [Type] An unsigned 64-bit integer type
/// @!method self.float32
///   @return [Type] A 32-bit floating point type
/// @!method self.float64
///   @return [Type] A 64-bit floating point type
/// @!method self.char
///   @return [Type] A Unicode character type
/// @!method self.string
///   @return [Type] A UTF-8 string type
/// @!method self.list(element_type)
///   @param element_type [Type] The type of list elements
///   @return [Type] A list type
/// @!method self.record(fields)
///   @param fields [Hash<String, Type>] A hash of field names to types
///   @return [Type] A record (struct) type
/// @!method self.tuple(types)
///   @param types [Array<Type>] The types in the tuple
///   @return [Type] A tuple type
/// @!method self.variant(cases)
///   @param cases [Hash<String, Type|nil>] A hash of case names to optional types
///   @return [Type] A variant type
/// @!method self.enum(cases)
///   @param cases [Array<String>] The enum case names
///   @return [Type] An enum type
/// @!method self.option(inner_type)
///   @param inner_type [Type] The type of the optional value
///   @return [Type] An option type
/// @!method self.result(ok_type, err_type)
///   @param ok_type [Type, nil] The type of the ok variant (nil for result<_, E>)
///   @param err_type [Type, nil] The type of the error variant (nil for result<T, _>)
///   @return [Type] A result type
/// @!method self.flags(flag_names)
///   @param flag_names [Array<String>] The flag names
///   @return [Type] A flags type
#[derive(Clone, TypedData)]
#[magnus(class = "Wasmtime::Component::Type", free_immediately)]
pub struct RbComponentType {
    inner: ComponentType,
}

impl magnus::DataTypeFunctions for RbComponentType {}

impl RbComponentType {
    pub fn new(inner: ComponentType) -> Self {
        Self { inner }
    }

    /// @yard
    /// Wrap a Ruby value with type information for use as a host function return value.
    /// @def wrap(value)
    /// @param value [Object] The Ruby value to wrap
    /// @return [WrappedValue] A wrapped value that can be returned from host functions
    pub fn wrap(&self, value: Value) -> WrappedValue {
        WrappedValue::new(value, self.inner.clone())
    }
}

/// @yard
/// @rename Wasmtime::Component::WrappedValue
/// A Ruby value wrapped with component model type information.
/// Returned from {Type#wrap} and used as host function return values.
#[derive(Clone, TypedData)]
#[magnus(class = "Wasmtime::Component::WrappedValue", mark, free_immediately)]
pub struct WrappedValue {
    value: Opaque<Value>,
    component_type: ComponentType,
}
unsafe impl Send for WrappedValue {}

impl magnus::DataTypeFunctions for WrappedValue {
    fn mark(&self, marker: &magnus::gc::Marker) {
        marker.mark(self.value);
    }
}

impl WrappedValue {
    pub fn new(value: Value, component_type: ComponentType) -> Self {
        Self {
            value: value.into(),
            component_type,
        }
    }

    /// Get the wrapped Ruby value
    pub fn value(&self, ruby: &Ruby) -> Value {
        ruby.get_inner(self.value)
    }

    /// Get the component type information
    pub fn component_type(&self) -> &ComponentType {
        &self.component_type
    }
}

pub struct TypeFactory;

impl TypeFactory {
    pub fn bool(_ruby: &Ruby) -> RbComponentType {
        RbComponentType::new(ComponentType::Bool)
    }

    pub fn s8(_ruby: &Ruby) -> RbComponentType {
        RbComponentType::new(ComponentType::S8)
    }

    pub fn u8(_ruby: &Ruby) -> RbComponentType {
        RbComponentType::new(ComponentType::U8)
    }

    pub fn s16(_ruby: &Ruby) -> RbComponentType {
        RbComponentType::new(ComponentType::S16)
    }

    pub fn u16(_ruby: &Ruby) -> RbComponentType {
        RbComponentType::new(ComponentType::U16)
    }

    pub fn s32(_ruby: &Ruby) -> RbComponentType {
        RbComponentType::new(ComponentType::S32)
    }

    pub fn u32(_ruby: &Ruby) -> RbComponentType {
        RbComponentType::new(ComponentType::U32)
    }

    pub fn s64(_ruby: &Ruby) -> RbComponentType {
        RbComponentType::new(ComponentType::S64)
    }

    pub fn u64(_ruby: &Ruby) -> RbComponentType {
        RbComponentType::new(ComponentType::U64)
    }

    pub fn float32(_ruby: &Ruby) -> RbComponentType {
        RbComponentType::new(ComponentType::Float32)
    }

    pub fn float64(_ruby: &Ruby) -> RbComponentType {
        RbComponentType::new(ComponentType::Float64)
    }

    pub fn char(_ruby: &Ruby) -> RbComponentType {
        RbComponentType::new(ComponentType::Char)
    }

    pub fn string(_ruby: &Ruby) -> RbComponentType {
        RbComponentType::new(ComponentType::String)
    }

    pub fn list(_ruby: &Ruby, element_type: &RbComponentType) -> RbComponentType {
        RbComponentType::new(ComponentType::List(Box::new(element_type.inner.clone())))
    }

    pub fn record(_ruby: &Ruby, fields: RHash) -> Result<RbComponentType, Error> {
        let mut record_fields = Vec::new();

        // Use foreach to iterate over hash
        fields.foreach(|key: Value, ty_value: Value| {
            let name = RString::try_convert(key)?.to_string()?;
            let ty_ref: &RbComponentType = TryConvert::try_convert(ty_value)?;
            record_fields.push(RecordField {
                name,
                ty: ty_ref.inner.clone(),
            });
            Ok(ForEach::Continue)
        })?;

        Ok(RbComponentType::new(ComponentType::Record(record_fields)))
    }

    pub fn tuple(_ruby: &Ruby, types: RArray) -> Result<RbComponentType, Error> {
        let mut tuple_types = Vec::with_capacity(types.len());

        for ty_value in unsafe { types.as_slice() } {
            let ty_ref: &RbComponentType = TryConvert::try_convert(*ty_value)?;
            tuple_types.push(ty_ref.inner.clone());
        }

        Ok(RbComponentType::new(ComponentType::Tuple(tuple_types)))
    }

    pub fn variant(_ruby: &Ruby, cases: RHash) -> Result<RbComponentType, Error> {
        let mut variant_cases = Vec::new();

        // Use foreach to iterate over hash
        cases.foreach(|key: Value, ty_value: Value| {
            let name = RString::try_convert(key)?.to_string()?;
            let ty = if ty_value.is_nil() {
                None
            } else {
                let ty_ref: &RbComponentType = TryConvert::try_convert(ty_value)?;
                Some(ty_ref.inner.clone())
            };
            variant_cases.push(VariantCase { name, ty });
            Ok(ForEach::Continue)
        })?;

        Ok(RbComponentType::new(ComponentType::Variant(variant_cases)))
    }

    pub fn enum_type(_ruby: &Ruby, cases: RArray) -> Result<RbComponentType, Error> {
        let mut enum_cases = Vec::with_capacity(cases.len());

        for case_value in unsafe { cases.as_slice() } {
            let case_name = RString::try_convert(*case_value)?.to_string()?;
            enum_cases.push(case_name);
        }

        Ok(RbComponentType::new(ComponentType::Enum(enum_cases)))
    }

    pub fn option(_ruby: &Ruby, inner_type: &RbComponentType) -> RbComponentType {
        RbComponentType::new(ComponentType::Option(Box::new(inner_type.inner.clone())))
    }

    pub fn result(
        _ruby: &Ruby,
        ok_type: Option<&RbComponentType>,
        err_type: Option<&RbComponentType>,
    ) -> RbComponentType {
        RbComponentType::new(ComponentType::Result {
            ok: ok_type.map(|t| Box::new(t.inner.clone())),
            err: err_type.map(|t| Box::new(t.inner.clone())),
        })
    }

    pub fn flags(_ruby: &Ruby, flag_names: RArray) -> Result<RbComponentType, Error> {
        let mut flags = Vec::with_capacity(flag_names.len());

        for flag_value in unsafe { flag_names.as_slice() } {
            let flag_name = RString::try_convert(*flag_value)?.to_string()?;
            flags.push(flag_name);
        }

        Ok(RbComponentType::new(ComponentType::Flags(flags)))
    }
}

pub fn init(ruby: &Ruby, namespace: &magnus::RModule) -> Result<(), Error> {
    let type_class = namespace.define_class("Type", ruby.class_object())?;

    // Factory methods
    type_class.define_singleton_method("bool", function!(TypeFactory::bool, 0))?;
    type_class.define_singleton_method("s8", function!(TypeFactory::s8, 0))?;
    type_class.define_singleton_method("u8", function!(TypeFactory::u8, 0))?;
    type_class.define_singleton_method("s16", function!(TypeFactory::s16, 0))?;
    type_class.define_singleton_method("u16", function!(TypeFactory::u16, 0))?;
    type_class.define_singleton_method("s32", function!(TypeFactory::s32, 0))?;
    type_class.define_singleton_method("u32", function!(TypeFactory::u32, 0))?;
    type_class.define_singleton_method("s64", function!(TypeFactory::s64, 0))?;
    type_class.define_singleton_method("u64", function!(TypeFactory::u64, 0))?;
    type_class.define_singleton_method("float32", function!(TypeFactory::float32, 0))?;
    type_class.define_singleton_method("float64", function!(TypeFactory::float64, 0))?;
    type_class.define_singleton_method("char", function!(TypeFactory::char, 0))?;
    type_class.define_singleton_method("string", function!(TypeFactory::string, 0))?;
    type_class.define_singleton_method("list", function!(TypeFactory::list, 1))?;
    type_class.define_singleton_method("record", function!(TypeFactory::record, 1))?;
    type_class.define_singleton_method("tuple", function!(TypeFactory::tuple, 1))?;
    type_class.define_singleton_method("variant", function!(TypeFactory::variant, 1))?;
    type_class.define_singleton_method("enum", function!(TypeFactory::enum_type, 1))?;
    type_class.define_singleton_method("option", function!(TypeFactory::option, 1))?;
    type_class.define_singleton_method("result", function!(TypeFactory::result, 2))?;
    type_class.define_singleton_method("flags", function!(TypeFactory::flags, 1))?;

    // Instance method for wrapping values
    type_class.define_method("wrap", method!(RbComponentType::wrap, 1))?;

    // WrappedValue class
    let _wrapped_value_class = namespace.define_class("WrappedValue", ruby.class_object())?;

    Ok(())
}
