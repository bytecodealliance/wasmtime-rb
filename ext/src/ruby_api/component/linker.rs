use super::root;
use crate::ruby_api::store::{Store, StoreContextValue};

use crate::error;
use magnus::{
    class, function, gc::Marker, method, r_string::RString, scan_args, typed_data::Obj,
    DataTypeFunctions, Error, Module, Object, Ruby, TypedData, Value,
};

pub fn init(_ruby: &Ruby, namespace: &RModule) -> Result<(), Error> {
    let _class = namespace.define_class("Linker", class::object())?;

    Ok(())
}
