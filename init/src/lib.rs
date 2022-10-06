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
use core::pin::Pin;

use pin::PinnedUninit;
pub use ptr::{Init, Uninit};
pub mod pin;

pub mod iter;

pub mod slice;

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
pub trait TryPinInitialize<T: ?Sized> {
    /// the error reported by this
    type Error;

    /// attempt to initialize the pointer
    ///
    /// if this function returns Ok, then the ptr was initialized
    /// otherwise, then the ptr may not be initialized
    fn try_pin_init(self, ptr: PinnedUninit<T>) -> Result<Pin<Init<T>>, Self::Error>;
}

/// A trait to initialize a T
pub trait Initialize<T: ?Sized>: TryInitialize<T, Error = core::convert::Infallible> {
    /// initializes the ptr
    fn init(self, ptr: Uninit<T>) -> Init<T>;
}

/// A trait to initialize a T
pub trait PinInitialize<T: ?Sized>: TryPinInitialize<T, Error = core::convert::Infallible> {
    /// attempt to initialize the pointer
    ///
    /// if this function returns Ok, then the ptr was initialized
    /// otherwise, then the ptr may not be initialized
    fn pin_init(self, ptr: PinnedUninit<T>) -> Pin<Init<T>>;
}

impl<F: FnOnce(Uninit<T>) -> Result<Init<T>, E>, E, T: ?Sized> TryInitialize<T> for F {
    type Error = E;

    #[inline]
    fn try_init(self, ptr: Uninit<T>) -> Result<Init<T>, Self::Error> {
        self(ptr)
    }
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

impl<F: FnOnce(PinnedUninit<T>) -> Result<Pin<Init<T>>, E>, E, T: ?Sized> TryPinInitialize<T>
    for F
{
    type Error = E;

    #[inline]
    fn try_pin_init(self, ptr: PinnedUninit<T>) -> Result<Pin<Init<T>>, Self::Error> {
        self(ptr)
    }
}

impl<T, I: TryPinInitialize<T, Error = core::convert::Infallible>> PinInitialize<T> for I {
    fn pin_init(self, ptr: PinnedUninit<T>) -> Pin<Init<T>> {
        match self.try_pin_init(ptr) {
            Ok(init) => init,
            Err(err) => match err {},
        }
    }
}

/// A function which will initialize without error
#[derive(Debug, Clone, Copy)]
pub struct InitFn<F> {
    func: F,
}

impl<F> InitFn<F> {
    /// Create a new initializer for infallible functions
    #[inline]
    pub fn new<T: ?Sized>(func: F) -> Self
    where
        F: FnOnce(Uninit<T>) -> Init<T>,
    {
        Self { func }
    }
}

impl<F: FnOnce(Uninit<T>) -> Init<T>, T: ?Sized> TryInitialize<T> for InitFn<F> {
    type Error = core::convert::Infallible;

    #[inline]
    fn try_init(self, ptr: Uninit<T>) -> Result<Init<T>, Self::Error> {
        Ok((self.func)(ptr))
    }
}

/// A function which will initialize without error
#[derive(Debug, Clone, Copy)]
pub struct TryInitFn<F> {
    func: F,
}

impl<F> TryInitFn<F> {
    /// Create a new initializer for infallible functions
    #[inline]
    pub fn new<T: ?Sized, E>(func: F) -> Self
    where
        F: FnOnce(Uninit<T>) -> Result<Init<T>, E>,
    {
        Self { func }
    }
}

impl<F: FnOnce(Uninit<T>) -> Result<Init<T>, E>, E, T: ?Sized> TryInitialize<T> for TryInitFn<F> {
    type Error = E;

    #[inline]
    fn try_init(self, ptr: Uninit<T>) -> Result<Init<T>, Self::Error> {
        (self.func)(ptr)
    }
}

/// A function which will initialize without error
#[derive(Debug, Clone, Copy)]
pub struct PinInitFn<F> {
    func: F,
}

impl<F> PinInitFn<F> {
    /// Create a new initializer for infallible functions
    #[inline]
    pub fn new<T: ?Sized>(func: F) -> Self
    where
        F: FnOnce(PinnedUninit<T>) -> Pin<Init<T>>,
    {
        Self { func }
    }
}

impl<F: FnOnce(PinnedUninit<T>) -> Pin<Init<T>>, T: ?Sized> TryPinInitialize<T> for PinInitFn<F> {
    type Error = core::convert::Infallible;

    #[inline]
    fn try_pin_init(self, ptr: PinnedUninit<T>) -> Result<Pin<Init<T>>, Self::Error> {
        Ok((self.func)(ptr))
    }
}

/// A function which will initialize without error
#[derive(Debug, Clone, Copy)]
pub struct TryPinInitFn<F> {
    func: F,
}

impl<F> TryPinInitFn<F> {
    /// Create a new initializer for infallible functions
    #[inline]
    pub fn new<T: ?Sized, E>(func: F) -> Self
    where
        F: FnOnce(PinnedUninit<T>) -> Result<Pin<Init<T>>, E>,
    {
        Self { func }
    }
}

impl<F: FnOnce(PinnedUninit<T>) -> Result<Pin<Init<T>>, E>, E, T: ?Sized> TryPinInitialize<T>
    for TryPinInitFn<F>
{
    type Error = E;

    #[inline]
    fn try_pin_init(self, ptr: PinnedUninit<T>) -> Result<Pin<Init<T>>, Self::Error> {
        (self.func)(ptr)
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
