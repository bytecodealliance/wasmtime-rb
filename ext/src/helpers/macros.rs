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

/// Boilerplate for defining send and sync on a Magnus struct when the feature is enabled.
#[macro_export]
macro_rules! unsafe_impl_send_sync {
    ($struct:ident) => {
        #[cfg(feature = "unsafe-impl-send")]
        unsafe impl Send for $struct {}
        #[cfg(feature = "unsafe-impl-sync")]
        unsafe impl Sync for $struct {}
    };
    ($struct:ident <'_>) => {
        #[cfg(feature = "unsafe-impl-send")]
        unsafe impl Send for $struct<'_> {}
        #[cfg(feature = "unsafe-impl-sync")]
        unsafe impl Sync for $struct<'_> {}
    };
}
