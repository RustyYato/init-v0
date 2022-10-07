//! the fundamental traits that underpin this crate

use core::pin::Pin;

use crate::{pin::PinnedUninit, Init, Uninit};

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

impl<T: ?Sized, I: TryInitialize<T, Error = core::convert::Infallible>> Initialize<T> for I {
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

impl<T: ?Sized, I: TryPinInitialize<T, Error = core::convert::Infallible>> PinInitialize<T> for I {
    fn pin_init(self, ptr: PinnedUninit<T>) -> Pin<Init<T>> {
        match self.try_pin_init(ptr) {
            Ok(init) => init,
            Err(err) => match err {},
        }
    }
}
