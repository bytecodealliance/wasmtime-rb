use std::{
    ffi::c_void,
    mem::MaybeUninit,
    panic::{self, AssertUnwindSafe},
    ptr::null_mut,
    thread,
};

use rb_sys::{rb_thread_call_with_gvl, rb_thread_call_without_gvl, ruby_thread_has_gvl_p};

#[inline]
fn has_gvl() -> bool {
    unsafe { ruby_thread_has_gvl_p() != 0 }
}

unsafe extern "C" fn call_trampoline<F, R>(arg: *mut c_void) -> *mut c_void
where
    F: FnOnce() -> R,
{
    let data = unsafe { &mut *(arg as *mut (Option<F>, MaybeUninit<thread::Result<R>>)) };
    let func = data.0.take().expect("closure called more than once");
    data.1.write(panic::catch_unwind(AssertUnwindSafe(func)));
    null_mut()
}

pub fn nogvl<F, R>(func: F) -> R
where
    F: FnOnce() -> R,
{
    let mut data: (Option<F>, MaybeUninit<thread::Result<R>>) = (Some(func), MaybeUninit::uninit());
    let arg = &mut data as *mut _ as *mut c_void;

    unsafe {
        rb_thread_call_without_gvl(Some(call_trampoline::<F, R>), arg, None, null_mut());
        data.1
            .assume_init()
            .unwrap_or_else(|e| panic::resume_unwind(e))
    }
}

pub fn with_gvl<F, R>(func: F) -> R
where
    F: FnOnce() -> R,
{
    if has_gvl() {
        return func();
    }

    let mut data: (Option<F>, MaybeUninit<thread::Result<R>>) = (Some(func), MaybeUninit::uninit());
    let arg = &mut data as *mut _ as *mut c_void;

    unsafe {
        rb_thread_call_with_gvl(Some(call_trampoline::<F, R>), arg);
        data.1
            .assume_init()
            .unwrap_or_else(|e| panic::resume_unwind(e))
    }
}
