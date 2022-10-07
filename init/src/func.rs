//! combinators that allow writing custom initializers

use core::pin::Pin;

use crate::{
    pin_uninit::PinnedUninit,
    traits::{TryInitialize, TryPinInitialize},
    Init, Uninit,
};

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
