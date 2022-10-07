//! sugary functions to initialize values behind raw pointers

use core::pin::Pin;

use crate::{
    pin::PinnedUninit,
    traits::{Initialize, PinInitialize, TryInitialize, TryPinInitialize},
    Uninit,
};

/// Try to initialize the ptr in place
///
/// if this function returns Ok, then the ptr was initialized
/// otherwise, then the ptr may not be initialized
///
/// # Safety
///
/// The pointer must be
/// * non-null
/// * allocated for T's layout
/// * writable for T's layout
/// * readable for T's layout after written to
#[inline]
pub unsafe fn try_init_in_place<T: ?Sized, I: TryInitialize<T>>(
    init: I,
    ptr: *mut T,
) -> Result<(), I::Error> {
    // SAFETY: the from_raw safety checks are forwarded to `try_init_in_place`
    match init.try_init(unsafe { Uninit::from_raw(ptr) }) {
        Err(err) => Err(err),
        Ok(init) => {
            let old = init.into_raw();
            debug_assert_eq!(
                ptr,
                old.as_ptr(),
                "SOUNDNESS BUG: try_init was able to return \
            a different pointer than it was passed in which amounts ot a \
            soundness bug either in the implementation of I::try_init if \
            unsafe was used or the init crate"
            );
            Ok(())
        }
    }
}

/// Try to initialize the ptr in place
///
/// if this function returns normally, then the ptr was initialized
/// otherwise, then the ptr may not be initialized
///
/// # Safety
///
/// The pointer must be
/// * non-null
/// * allocated for T's layout
/// * writable for T's layout
/// * readable for T's layout after written to
#[inline]
pub unsafe fn init_in_place<T: ?Sized, I: Initialize<T>>(init: I, ptr: *mut T) {
    // SAFETY: the from_raw safety checks are forwarded to `try_init_in_place`
    match init.try_init(unsafe { Uninit::from_raw(ptr) }) {
        Err(err) => match err {},
        Ok(init) => {
            let old = init.into_raw();
            debug_assert_eq!(
                ptr,
                old.as_ptr(),
                "SOUNDNESS BUG: try_init was able to return \
            a different pointer than it was passed in which amounts ot a \
            soundness bug either in the implementation of I::try_init if \
            unsafe was used or the init crate"
            );
        }
    }
}

/// Try to initialize the ptr in place
///
/// if this function returns Ok, then the ptr was initialized
/// otherwise, then the ptr may not be initialized
///
/// # Safety
///
/// The pointee must be pinned
///
/// The pointer must be
/// * non-null
/// * allocated for T's layout
/// * writable for T's layout
/// * readable for T's layout after written to
#[inline]
pub unsafe fn try_pin_init_in_place<T: ?Sized, I: TryPinInitialize<T>>(
    init: I,
    ptr: *mut T,
) -> Result<(), I::Error> {
    // SAFETY: the from_raw safety checks are forwarded to `try_init_in_place`
    let uninit = unsafe { Uninit::from_raw(ptr) };
    // SAFETY: the pointee is pinned
    let uninit = unsafe { PinnedUninit::new_unchecked(uninit) };
    match init.try_pin_init(uninit) {
        Err(err) => Err(err),
        Ok(init) => {
            // SAFETY: the pointee is untouched
            let init = unsafe { Pin::into_inner_unchecked(init) };
            let old = init.into_raw();
            debug_assert_eq!(
                ptr,
                old.as_ptr(),
                "SOUNDNESS BUG: try_init was able to return \
            a different pointer than it was passed in which amounts ot a \
            soundness bug either in the implementation of I::try_init if \
            unsafe was used or the init crate"
            );
            Ok(())
        }
    }
}

/// Try to initialize the ptr in place
///
/// if this function returns normally, then the ptr was initialized
/// otherwise, then the ptr may not be initialized
///
/// # Safety
///
/// The pointee must be pinned
///
/// The pointer must be
/// * non-null
/// * allocated for T's layout
/// * writable for T's layout
/// * readable for T's layout after written to
#[inline]
pub unsafe fn pin_init_in_place<T: ?Sized, I: PinInitialize<T>>(init: I, ptr: *mut T) {
    // SAFETY: the from_raw safety checks are forwarded to `try_init_in_place`
    let uninit = unsafe { Uninit::from_raw(ptr) };
    // SAFETY: the pointee is pinned
    let uninit = unsafe { PinnedUninit::new_unchecked(uninit) };
    match init.try_pin_init(uninit) {
        Err(err) => match err {},
        Ok(init) => {
            // SAFETY: the pointee is untouched
            let init = unsafe { Pin::into_inner_unchecked(init) };
            let old = init.into_raw();
            debug_assert_eq!(
                ptr,
                old.as_ptr(),
                "SOUNDNESS BUG: try_init was able to return \
            a different pointer than it was passed in which amounts ot a \
            soundness bug either in the implementation of I::try_init if \
            unsafe was used or the init crate"
            );
        }
    }
}
