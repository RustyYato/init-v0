//! Init is a crate that handles fallible in-place initialization

#![feature(slice_ptr_len, dropck_eyepatch)]
#![forbid(
    clippy::undocumented_unsafe_blocks,
    clippy::missing_safety_doc,
    unsafe_op_in_unsafe_fn,
    missing_docs,
    clippy::missing_panics_doc
)]
#![no_std]

mod ptr;
pub use ptr::{Init, Uninit};

/// A trait to initialize a T
pub trait TryInitialize<T: ?Sized> {
    /// the error reported by this
    type Error;

    /// attempt to initialize the pointer
    ///
    /// if this function returns Ok, then the ptr was initialized
    /// otherwise, then the ptr may not be initialized
    fn try_init(self, ptr: Uninit<T>) -> Result<Init<T>, Self::Error>;
}

/// A trait to initialize a T
pub trait Initialize<T: ?Sized>: TryInitialize<T, Error = core::convert::Infallible> {
    /// initializes the ptr
    fn init(self, ptr: Uninit<T>) -> Init<T>;
}

impl<T, I: TryInitialize<T, Error = core::convert::Infallible>> Initialize<T> for I {
    #[inline]
    fn init(self, ptr: Uninit<T>) -> Init<T> {
        match self.try_init(ptr) {
            Ok(init) => init,
            Err(err) => match err {},
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
