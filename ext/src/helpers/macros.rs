/// A macro to define a new `Id` const for a given string.
#[macro_export]
macro_rules! define_rb_intern {
    ($($name:ident => $id:expr,)*) => {
        $(
            lazy_static::lazy_static! {
                /// Define a Ruby internal `Id`. Equivalent to `rb_intern("$name")`
                pub static ref $name: $crate::helpers::StaticId = $crate::helpers::StaticId::intern_str($id);
            }
        )*
    };
}

#[macro_export]
macro_rules! define_data_class {
    ($namespace:expr, $name:expr, $parent:expr) => {{
        let class = $namespace.define_class($name, $parent).unwrap();
        magnus::Class::undef_alloc_func(class);
        class
    }};
    ($namespace:expr, $name:expr) => {
        define_data_class!($namespace, $name, magnus::RClass::default())
    };
}
