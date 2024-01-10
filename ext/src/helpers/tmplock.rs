use magnus::{
    rb_sys::{protect, AsRawValue},
    RString,
};

pub trait Tmplock {
    unsafe fn as_locked_slice(&self) -> Result<(&[u8], TmplockGuard), magnus::Error>;
    unsafe fn as_locked_str(&self) -> Result<(&str, TmplockGuard), magnus::Error>;
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
    unsafe fn as_locked_slice(&self) -> Result<(&[u8], TmplockGuard), magnus::Error> {
        let raw = self.as_raw();
        let slice = self.as_slice();
        let raw = protect(|| rb_sys::rb_str_locktmp(raw))?;
        let guard = TmplockGuard { raw };

        Ok((slice, guard))
    }

    unsafe fn as_locked_str(&self) -> Result<(&str, TmplockGuard), magnus::Error> {
        let str_result = self.as_str()?;
        let raw = self.as_raw();
        let raw = protect(|| rb_sys::rb_str_locktmp(raw))?;
        let guard = TmplockGuard { raw };

        Ok((str_result, guard))
    }
}
