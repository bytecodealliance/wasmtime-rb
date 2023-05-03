use magnus::Error;
mod helpers;
mod ruby_api;

#[cfg(feature = "ruby-api")]
pub use ruby_api::*;

#[cfg(not(feature = "ruby-api"))]
pub(crate) use ruby_api::*;

#[cfg(all(feature = "global-tracking-allocator", not(feature = "ruby-api")))]
rb_sys::set_global_tracking_allocator!();

#[magnus::init]
pub fn init() -> Result<(), Error> {
    #[cfg(ruby_gte_3_0)]
    unsafe {
        rb_sys::rb_ext_ractor_safe(true);
    }
    ruby_api::init()
}
