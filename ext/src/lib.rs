use magnus::Error;
mod helpers;
mod ruby_api;

#[cfg(feature = "ruby-api")]
pub use ruby_api::*;

#[cfg(not(feature = "ruby-api"))]
pub(crate) use ruby_api::*;

#[magnus::init]
pub fn init() -> Result<(), Error> {
    ruby_api::init()
}
