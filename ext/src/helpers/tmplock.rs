use magnus::{
    rb_sys::{protect, AsRawValue},
    value::ReprValue,
    RString,
};

pub trait Tmplock {
    fn as_locked_slice(&self) -> Result<(&[u8], Option<TmplockGuard>), magnus::Error>;
    fn as_locked_str(&self) -> Result<(&str, Option<TmplockGuard>), magnus::Error>;
}

#[derive(Debug)]
#[repr(transparent)]
pub struct TmplockGuard {
    raw: rb_sys::VALUE,
}

impl Drop for TmplockGuard {
    fn drop(&mut self) {
        let result = unsafe { protect(|| rb_sys::rb_str_unlocktmp(self.raw)) };
        debug_assert!(
            result.is_ok(),
            "failed to unlock tmplock for unknown reason"
        );
    }
}

impl Tmplock for RString {
    fn as_locked_slice(&self) -> Result<(&[u8], Option<TmplockGuard>), magnus::Error> {
        let raw = self.as_raw();
        let slice = unsafe { self.as_slice() };
        let guard = if self.is_frozen() {
            None
        } else {
            let raw = protect(|| unsafe { rb_sys::rb_str_locktmp(raw) })?;
            Some(TmplockGuard { raw })
        };

        Ok((slice, guard))
    }

    fn as_locked_str(&self) -> Result<(&str, Option<TmplockGuard>), magnus::Error> {
        let str_result = unsafe { self.as_str()? };
        let guard = if self.is_frozen() {
            None
        } else {
            let raw = self.as_raw();
            let raw = protect(|| unsafe { rb_sys::rb_str_locktmp(raw) })?;
            Some(TmplockGuard { raw })
        };

        Ok((str_result, guard))
    }
}
