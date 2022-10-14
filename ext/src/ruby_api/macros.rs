/// A macro to define a new `Id` const for a given string.
#[macro_export]
macro_rules! define_rb_intern {
    ($($name:ident => $id:expr,)*) => {
        $(
            lazy_static::lazy_static! {
                /// Define a Ruby internal `Id`. Equivalent to `rb_intern("$name")`
                pub static ref $name: $crate::ruby_api::static_id::StaticId = $crate::ruby_api::static_id::StaticId::intern_str($id);
            }
        )*
    };
}
