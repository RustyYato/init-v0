//! A combinator to convert from a `TryInitialize` to a `TryPinInitialize` and vice versa

use core::pin::Pin;

use crate::{
    pin_uninit::PinnedUninit,
    traits::{TryInitialize, TryPinInitialize},
    Init, Uninit,
};

/// a combinator created from [`TryInitialize::to_pin_init`]
pub struct AsPinInit<I> {
    init: I,
}

impl<I> AsPinInit<I> {
    /// Create a new AsPinInit
    #[inline(always)]
    pub fn new(init: I) -> Self {
        Self { init }
    }
}

impl<I: TryInitialize<T>, T: ?Sized> TryInitialize<T> for AsPinInit<I> {
    type Error = I::Error;

    #[inline]
    fn try_init(self, ptr: Uninit<T>) -> Result<Init<T>, Self::Error> {
        self.init.try_init(ptr)
    }
}

impl<I: TryInitialize<T>, T: ?Sized + Unpin> TryPinInitialize<T> for AsPinInit<I> {
    type Error = I::Error;

    #[inline]
    fn try_pin_init(self, ptr: PinnedUninit<T>) -> Result<Pin<Init<T>>, Self::Error> {
        match self.init.try_init(ptr.into_inner()) {
            Ok(init) => Ok(Pin::new(init)),
            Err(_) => todo!(),
        }
    }
}

/// a combinator created from [`TryInitialize::to_pin_init`]
pub struct AsInit<I> {
    init: I,
}

impl<I> AsInit<I> {
    /// Create a new AsPinInit
    #[inline(always)]
    pub fn new(init: I) -> Self {
        Self { init }
    }
}

impl<I: TryPinInitialize<T>, T: ?Sized + Unpin> TryInitialize<T> for AsInit<I> {
    type Error = I::Error;

    #[inline]
    fn try_init(self, ptr: Uninit<T>) -> Result<Init<T>, Self::Error> {
        match self.init.try_pin_init(PinnedUninit::new(ptr)) {
            Ok(init) => Ok(Pin::into_inner(init)),
            Err(err) => Err(err),
        }
    }
}

impl<I: TryPinInitialize<T>, T: ?Sized> TryPinInitialize<T> for AsInit<I> {
    type Error = I::Error;

    #[inline]
    fn try_pin_init(self, ptr: PinnedUninit<T>) -> Result<Pin<Init<T>>, Self::Error> {
        self.init.try_pin_init(ptr)
    }
}
