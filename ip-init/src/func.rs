//! combinators that allow writing custom initializers

use core::{marker::PhantomData, pin::Pin};

use crate::{
    pin_uninit::PinnedUninit,
    traits::{TryInitialize, TryPinInitialize},
    Init, Uninit,
};

/// A function which will initialize without error
#[derive(Debug, Clone, Copy)]
pub struct InitFn<F, T: ?Sized> {
    func: F,
    _ty: PhantomData<fn() -> T>,
}

impl<F, T: ?Sized> InitFn<F, T> {
    /// Create a new initializer for infallible functions
    #[inline]
    pub fn new(func: F) -> Self
    where
        F: FnOnce(Uninit<T>) -> Init<T>,
    {
        Self {
            func,
            _ty: PhantomData,
        }
    }
}

impl<F: FnOnce(Uninit<T>) -> Init<T>, T: ?Sized> TryInitialize<T> for InitFn<F, T> {
    type Error = core::convert::Infallible;

    #[inline]
    fn try_init(self, ptr: Uninit<T>) -> Result<Init<T>, Self::Error> {
        Ok((self.func)(ptr))
    }
}

/// A function which may initialize with error
#[derive(Debug, Clone, Copy)]
pub struct TryInitFn<F, T: ?Sized> {
    func: F,
    _ty: PhantomData<fn() -> T>,
}

impl<F, T: ?Sized> TryInitFn<F, T> {
    /// Create a new initializer for infallible functions
    #[inline]
    pub fn new<E>(func: F) -> Self
    where
        F: FnOnce(Uninit<T>) -> Result<Init<T>, E>,
    {
        Self {
            func,
            _ty: PhantomData,
        }
    }
}

impl<F: FnOnce(Uninit<T>) -> Result<Init<T>, E>, E, T: ?Sized> TryInitialize<T>
    for TryInitFn<F, T>
{
    type Error = E;

    #[inline]
    fn try_init(self, ptr: Uninit<T>) -> Result<Init<T>, Self::Error> {
        (self.func)(ptr)
    }
}

/// A function which will pin initialize without error
#[derive(Debug, Clone, Copy)]
pub struct PinInitFn<F, T: ?Sized> {
    func: F,
    _ty: PhantomData<fn() -> T>,
}

impl<F, T: ?Sized> PinInitFn<F, T> {
    /// Create a new initializer for infallible functions
    #[inline]
    pub fn new(func: F) -> Self
    where
        F: FnOnce(PinnedUninit<T>) -> Pin<Init<T>>,
    {
        Self {
            func,
            _ty: PhantomData,
        }
    }
}

impl<F: FnOnce(PinnedUninit<T>) -> Pin<Init<T>>, T: ?Sized> TryPinInitialize<T>
    for PinInitFn<F, T>
{
    type Error = core::convert::Infallible;

    #[inline]
    fn try_pin_init(self, ptr: PinnedUninit<T>) -> Result<Pin<Init<T>>, Self::Error> {
        Ok((self.func)(ptr))
    }
}

/// A function which may pin initialize with error
#[derive(Debug, Clone, Copy)]
pub struct TryPinInitFn<F, T: ?Sized> {
    func: F,
    _ty: PhantomData<fn() -> T>,
}

impl<F, T: ?Sized> TryPinInitFn<F, T> {
    /// Create a new initializer for infallible functions
    #[inline]
    pub fn new<E>(func: F) -> Self
    where
        F: FnOnce(PinnedUninit<T>) -> Result<Pin<Init<T>>, E>,
    {
        Self {
            func,
            _ty: PhantomData,
        }
    }
}

impl<F: FnOnce(PinnedUninit<T>) -> Result<Pin<Init<T>>, E>, E, T: ?Sized> TryPinInitialize<T>
    for TryPinInitFn<F, T>
{
    type Error = E;

    #[inline]
    fn try_pin_init(self, ptr: PinnedUninit<T>) -> Result<Pin<Init<T>>, Self::Error> {
        (self.func)(ptr)
    }
}
