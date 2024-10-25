#[allow(warnings)]
mod bindings;

use bindings::exports::resource;
use bindings::Guest;
use paste::paste;

macro_rules! id_function {
    ($wasm_ty:ident, $rust_ty:ty) => {
        paste! {
            fn [<id_ $wasm_ty>](v: $rust_ty) -> $rust_ty {
                v
            }
        }
    };
}

struct WrappedString(String);
impl resource::GuestWrappedString for WrappedString {
    fn new(v: String) -> Self {
        Self(v)
    }

    fn to_string(&self) -> String {
        self.0.clone()
    }
}

struct Component;
impl resource::Guest for Component {
    type WrappedString = WrappedString;

    fn resource_owned(_: resource::WrappedString) {}
}

impl Guest for Component {
    id_function!(bool, bool);
    id_function!(s8, i8);
    id_function!(u8, u8);
    id_function!(s16, i16);
    id_function!(u16, u16);
    id_function!(s32, i32);
    id_function!(u32, u32);
    id_function!(s64, i64);
    id_function!(u64, u64);
    id_function!(f32, f32);
    id_function!(f64, f64);
    id_function!(char, char);
    id_function!(string, String);
    id_function!(list, Vec<u32>);
    id_function!(record, bindings::Point);
    id_function!(tuple, (u32, String));
    id_function!(variant, bindings::Filter);
    id_function!(enum, bindings::Size);
    id_function!(option, Option<u32>);
    id_function!(result, Result<u32, u32>);
    id_function!(flags, bindings::Permission);
    id_function!(result_unit, Result<(), ()>);
}

bindings::export!(Component with_types_in bindings);
