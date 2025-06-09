#![allow(clippy::doc_lazy_continuation)]
use magnus::{Error, Ruby};
mod helpers;
mod ruby_api;

pub(crate) use ruby_api::*;

rb_sys::set_global_tracking_allocator!();

#[magnus::init]
pub fn init(ruby: &Ruby) -> Result<(), Error> {
    #[cfg(ruby_gte_3_0)]
    unsafe {
        rb_sys::rb_ext_ractor_safe(true);
    }
    ruby_api::init(ruby)
}
