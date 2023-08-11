use magnus::Error;
mod helpers;
mod ruby_api;

#[cfg(feature = "ruby-api")]
pub use ruby_api::*;

#[cfg(not(feature = "ruby-api"))]
pub(crate) use ruby_api::*;

#[cfg(not(feature = "ruby-api"))] // Let the upstream crate handle this
rb_sys::set_global_tracking_allocator!();

#[magnus::init]
pub fn init() -> Result<(), Error> {
    #[cfg(ruby_gte_3_0)]
    unsafe {
        rb_sys::rb_ext_ractor_safe(true);
    }
    ruby_api::init()
}
