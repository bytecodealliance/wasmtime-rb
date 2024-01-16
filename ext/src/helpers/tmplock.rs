use magnus::{
    rb_sys::{protect, AsRawValue},
    RString,
};

pub trait Tmplock {
    fn as_locked_slice(&self) -> Result<(&[u8], TmplockGuard), magnus::Error>;
    fn as_locked_str(&self) -> Result<(&str, TmplockGuard), magnus::Error>;
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
    fn as_locked_slice(&self) -> Result<(&[u8], TmplockGuard), magnus::Error> {
        let raw = self.as_raw();
        let slice = unsafe { self.as_slice() };
        let raw = protect(|| unsafe { rb_sys::rb_str_locktmp(raw) })?;
        let guard = TmplockGuard { raw };

        Ok((slice, guard))
    }

    fn as_locked_str(&self) -> Result<(&str, TmplockGuard), magnus::Error> {
        let str_result = unsafe { self.as_str()? };
        let raw = self.as_raw();
        let raw = protect(|| unsafe { rb_sys::rb_str_locktmp(raw) })?;
        let guard = TmplockGuard { raw };

        Ok((str_result, guard))
    }
}
