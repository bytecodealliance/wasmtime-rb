use magnus::Error;
mod helpers;
mod ruby_api;

pub use ruby_api::*;

#[magnus::init]
pub fn init() -> Result<(), Error> {
    ruby_api::init()
}
