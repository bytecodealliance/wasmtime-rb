use magnus::rb_sys::{AsRawId, FromRawId};
use magnus::{
    ruby_handle::RubyHandle,
    value::{Id, IntoId},
    Symbol,
};
use std::convert::TryInto;
use std::num::NonZeroUsize;

/// A static `Id` that can be used to refer to a Ruby ID.
///
/// Use `define_rb_intern!` to define it so that it will be cached in a global variable.
///
/// Magnus' `Id` can't be used for this purpose since it is not `Sync`, so cannot
/// be used as a global variable with `lazy_static` in `define_rb_intern!`.
/// See [this commit on the Magnus repo][commit].
///
/// [commit]: https://github.com/matsadler/magnus/commit/1a1c1ee874e15b0b222f7aae68bb9b5360072e57
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct StaticId(NonZeroUsize);

impl StaticId {
    // Use `define_rb_intern!` instead, which uses this function.
    pub fn intern_str(id: &'static str) -> Self {
        let id: Id = magnus::StaticSymbol::new(id).into();

        // SAFETY: Ruby will never return a `0` ID.
        StaticId(unsafe { NonZeroUsize::new_unchecked(id.as_raw() as _) })
    }
}

impl IntoId for StaticId {
    fn into_id_with(self, _: &RubyHandle) -> Id {
        // SAFEFY: This is safe because we know that the `Id` is something
        // returned from ruby.
        unsafe { Id::from_raw(self.0.get().try_into().expect("ID to be a usize")) }
    }
}

impl From<StaticId> for Symbol {
    fn from(static_id: StaticId) -> Self {
        let id: Id = static_id.into_id();
        id.into()
    }
}

impl std::cmp::PartialEq<Id> for StaticId {
    fn eq(&self, other: &Id) -> bool {
        other.as_raw() as usize == self.0.get()
    }
}
