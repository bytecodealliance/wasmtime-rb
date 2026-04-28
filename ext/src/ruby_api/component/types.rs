use crate::error;
use magnus::{
    class, function, prelude::*, r_hash::ForEach, Error, Module as _, RArray, RHash, RString, Ruby,
    Symbol, TryConvert, TypedData, Value,
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

    Ok(())
}

// Make ComponentType accessible from other component modules
pub(super) fn extract_component_type(value: Value) -> Result<ComponentType, Error> {
    let rb_ty: &RbComponentType = TryConvert::try_convert(value)?;
    Ok(rb_ty.inner.clone())
}

/// Convert wasmtime's component Type to our ComponentType
/// This is used for validating host function signatures against component imports
pub(super) fn wasmtime_type_to_component_type(
    ty: &wasmtime::component::Type,
) -> Result<ComponentType, String> {
    use wasmtime::component::types::ComponentItem;

    match ty {
        wasmtime::component::Type::Bool => Ok(ComponentType::Bool),
        wasmtime::component::Type::S8 => Ok(ComponentType::S8),
        wasmtime::component::Type::U8 => Ok(ComponentType::U8),
        wasmtime::component::Type::S16 => Ok(ComponentType::S16),
        wasmtime::component::Type::U16 => Ok(ComponentType::U16),
        wasmtime::component::Type::S32 => Ok(ComponentType::S32),
        wasmtime::component::Type::U32 => Ok(ComponentType::U32),
        wasmtime::component::Type::S64 => Ok(ComponentType::S64),
        wasmtime::component::Type::U64 => Ok(ComponentType::U64),
        wasmtime::component::Type::Float32 => Ok(ComponentType::Float32),
        wasmtime::component::Type::Float64 => Ok(ComponentType::Float64),
        wasmtime::component::Type::Char => Ok(ComponentType::Char),
        wasmtime::component::Type::String => Ok(ComponentType::String),
        wasmtime::component::Type::List(inner) => {
            let inner_ty = wasmtime_type_to_component_type(&inner.ty())?;
            Ok(ComponentType::List(Box::new(inner_ty)))
        }
        wasmtime::component::Type::Record(record) => {
            let mut fields = Vec::new();
            for field in record.fields() {
                let field_ty = wasmtime_type_to_component_type(&field.ty)?;
                fields.push(RecordField {
                    name: field.name.to_string(),
                    ty: field_ty,
                });
            }
            Ok(ComponentType::Record(fields))
        }
        wasmtime::component::Type::Tuple(tuple) => {
            let mut types = Vec::new();
            for ty in tuple.types() {
                types.push(wasmtime_type_to_component_type(&ty)?);
            }
            Ok(ComponentType::Tuple(types))
        }
        wasmtime::component::Type::Variant(variant) => {
            let mut cases = Vec::new();
            for case in variant.cases() {
                let case_ty = case
                    .ty
                    .as_ref()
                    .map(wasmtime_type_to_component_type)
                    .transpose()?;
                cases.push(VariantCase {
                    name: case.name.to_string(),
                    ty: case_ty,
                });
            }
            Ok(ComponentType::Variant(cases))
        }
        wasmtime::component::Type::Enum(enum_ty) => {
            let cases: Vec<String> = enum_ty.names().map(|s| s.to_string()).collect();
            Ok(ComponentType::Enum(cases))
        }
        wasmtime::component::Type::Option(opt) => {
            let inner_ty = wasmtime_type_to_component_type(&opt.ty())?;
            Ok(ComponentType::Option(Box::new(inner_ty)))
        }
        wasmtime::component::Type::Result(result) => {
            let ok = result
                .ok()
                .map(|ty| wasmtime_type_to_component_type(&ty).map(Box::new))
                .transpose()?;
            let err = result
                .err()
                .map(|ty| wasmtime_type_to_component_type(&ty).map(Box::new))
                .transpose()?;
            Ok(ComponentType::Result { ok, err })
        }
        wasmtime::component::Type::Flags(flags) => {
            let names: Vec<String> = flags.names().map(|s| s.to_string()).collect();
            Ok(ComponentType::Flags(names))
        }
        wasmtime::component::Type::Own(_) | wasmtime::component::Type::Borrow(_) => Err(
            "resource types (own/borrow) are not yet supported for host function validation"
                .to_string(),
        ),
        wasmtime::component::Type::Map(_) => {
            Err("map types are not yet supported for host function validation".to_string())
        }
        wasmtime::component::Type::Future(_) => {
            Err("future types are not yet supported for host function validation".to_string())
        }
        wasmtime::component::Type::Stream(_) => {
            Err("stream types are not yet supported for host function validation".to_string())
        }
        wasmtime::component::Type::ErrorContext => Err(
            "error-context types are not yet supported for host function validation".to_string(),
        ),
    }
}

/// Extract function parameter and result types from a ComponentFunc
pub(super) fn extract_func_types(
    func: &wasmtime::component::types::ComponentFunc,
) -> Result<(Vec<ComponentType>, Vec<ComponentType>), String> {
    let mut param_types = Vec::new();
    for (_name, ty) in func.params() {
        param_types.push(wasmtime_type_to_component_type(&ty)?);
    }

    let mut result_types = Vec::new();
    // Results is an iterator of Type directly (not tuples)
    for ty in func.results() {
        result_types.push(wasmtime_type_to_component_type(&ty)?);
    }

    Ok((param_types, result_types))
}

/// Compare two ComponentTypes for compatibility
/// Returns Ok(()) if types match, Err with descriptive message if they don't
pub(super) fn types_match(
    declared: &ComponentType,
    expected: &ComponentType,
) -> Result<(), String> {
    match (declared, expected) {
        // Primitive types must match exactly
        (ComponentType::Bool, ComponentType::Bool)
        | (ComponentType::S8, ComponentType::S8)
        | (ComponentType::U8, ComponentType::U8)
        | (ComponentType::S16, ComponentType::S16)
        | (ComponentType::U16, ComponentType::U16)
        | (ComponentType::S32, ComponentType::S32)
        | (ComponentType::U32, ComponentType::U32)
        | (ComponentType::S64, ComponentType::S64)
        | (ComponentType::U64, ComponentType::U64)
        | (ComponentType::Float32, ComponentType::Float32)
        | (ComponentType::Float64, ComponentType::Float64)
        | (ComponentType::Char, ComponentType::Char)
        | (ComponentType::String, ComponentType::String) => Ok(()),

        // List types: element types must match
        (ComponentType::List(d_inner), ComponentType::List(e_inner)) => {
            types_match(d_inner, e_inner).map_err(|e| format!("list element type mismatch: {}", e))
        }

        // Option types: inner types must match
        (ComponentType::Option(d_inner), ComponentType::Option(e_inner)) => {
            types_match(d_inner, e_inner).map_err(|e| format!("option type mismatch: {}", e))
        }

        // Record types: must have same fields with matching types
        (ComponentType::Record(d_fields), ComponentType::Record(e_fields)) => {
            if d_fields.len() != e_fields.len() {
                return Err(format!(
                    "record field count mismatch: declared has {} fields, expected has {}",
                    d_fields.len(),
                    e_fields.len()
                ));
            }

            for (d_field, e_field) in d_fields.iter().zip(e_fields.iter()) {
                if d_field.name != e_field.name {
                    return Err(format!(
                        "record field name mismatch: declared has '{}', expected has '{}'",
                        d_field.name, e_field.name
                    ));
                }
                types_match(&d_field.ty, &e_field.ty)
                    .map_err(|e| format!("record field '{}' type mismatch: {}", d_field.name, e))?;
            }
            Ok(())
        }

        // Tuple types: must have same number of elements with matching types
        (ComponentType::Tuple(d_types), ComponentType::Tuple(e_types)) => {
            if d_types.len() != e_types.len() {
                return Err(format!(
                    "tuple length mismatch: declared has {} elements, expected has {}",
                    d_types.len(),
                    e_types.len()
                ));
            }

            for (i, (d_ty, e_ty)) in d_types.iter().zip(e_types.iter()).enumerate() {
                types_match(d_ty, e_ty)
                    .map_err(|e| format!("tuple element {} mismatch: {}", i, e))?;
            }
            Ok(())
        }

        // Variant types: must have same cases with matching types
        (ComponentType::Variant(d_cases), ComponentType::Variant(e_cases)) => {
            if d_cases.len() != e_cases.len() {
                return Err(format!(
                    "variant case count mismatch: declared has {} cases, expected has {}",
                    d_cases.len(),
                    e_cases.len()
                ));
            }

            for (d_case, e_case) in d_cases.iter().zip(e_cases.iter()) {
                if d_case.name != e_case.name {
                    return Err(format!(
                        "variant case name mismatch: declared has '{}', expected has '{}'",
                        d_case.name, e_case.name
                    ));
                }

                match (&d_case.ty, &e_case.ty) {
                    (Some(d_ty), Some(e_ty)) => {
                        types_match(d_ty, e_ty).map_err(|e| {
                            format!("variant case '{}' type mismatch: {}", d_case.name, e)
                        })?;
                    }
                    (None, None) => {}
                    (Some(_), None) => {
                        return Err(format!(
                            "variant case '{}': declared has payload, expected has none",
                            d_case.name
                        ));
                    }
                    (None, Some(_)) => {
                        return Err(format!(
                            "variant case '{}': declared has no payload, expected has payload",
                            d_case.name
                        ));
                    }
                }
            }
            Ok(())
        }

        // Enum types: must have same cases in same order
        (ComponentType::Enum(d_cases), ComponentType::Enum(e_cases)) => {
            if d_cases.len() != e_cases.len() {
                return Err(format!(
                    "enum case count mismatch: declared has {} cases, expected has {}",
                    d_cases.len(),
                    e_cases.len()
                ));
            }

            for (d_case, e_case) in d_cases.iter().zip(e_cases.iter()) {
                if d_case != e_case {
                    return Err(format!(
                        "enum case mismatch: declared has '{}', expected has '{}'",
                        d_case, e_case
                    ));
                }
            }
            Ok(())
        }

        // Result types: ok and err types must match
        (
            ComponentType::Result {
                ok: d_ok,
                err: d_err,
            },
            ComponentType::Result {
                ok: e_ok,
                err: e_err,
            },
        ) => {
            match (d_ok, e_ok) {
                (Some(d_ty), Some(e_ty)) => {
                    types_match(d_ty, e_ty)
                        .map_err(|e| format!("result ok type mismatch: {}", e))?;
                }
                (None, None) => {}
                (Some(_), None) => {
                    return Err(
                        "result ok type mismatch: declared has ok type, expected has none"
                            .to_string(),
                    );
                }
                (None, Some(_)) => {
                    return Err(
                        "result ok type mismatch: declared has no ok type, expected has ok type"
                            .to_string(),
                    );
                }
            }

            match (d_err, e_err) {
                (Some(d_ty), Some(e_ty)) => {
                    types_match(d_ty, e_ty)
                        .map_err(|e| format!("result err type mismatch: {}", e))?;
                }
                (None, None) => {}
                (Some(_), None) => {
                    return Err(
                        "result err type mismatch: declared has err type, expected has none"
                            .to_string(),
                    );
                }
                (None, Some(_)) => {
                    return Err(
                        "result err type mismatch: declared has no err type, expected has err type"
                            .to_string(),
                    );
                }
            }
            Ok(())
        }

        // Flags types: must have same flags in same order
        (ComponentType::Flags(d_flags), ComponentType::Flags(e_flags)) => {
            if d_flags.len() != e_flags.len() {
                return Err(format!(
                    "flags count mismatch: declared has {} flags, expected has {}",
                    d_flags.len(),
                    e_flags.len()
                ));
            }

            for (d_flag, e_flag) in d_flags.iter().zip(e_flags.iter()) {
                if d_flag != e_flag {
                    return Err(format!(
                        "flag mismatch: declared has '{}', expected has '{}'",
                        d_flag, e_flag
                    ));
                }
            }
            Ok(())
        }

        // Type mismatch
        _ => Err(format!("expected {}, got {}", expected, declared)),
    }
}
