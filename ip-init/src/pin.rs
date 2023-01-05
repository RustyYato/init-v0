//! A combinator to convert from a [`TryInitialize`] to a [`TryPinInitialize`] and vice versa

use core::marker::PhantomData;

use crate::{
    pin_ptr::{PinnedInit, PinnedUninit},
    traits::{TryInitialize, TryPinInitialize},
    Init, Uninit,
};

/// a combinator created from [`TryInitialize::to_pin_init`]
pub struct AsPinInit<I, T: ?Sized> {
    init: I,
    _ty: PhantomData<fn() -> T>,
}

impl<I, T: ?Sized> AsPinInit<I, T> {
    /// Create a new AsPinInit
    #[inline(always)]
    pub fn new(init: I) -> Self {
        Self {
            init,
            _ty: PhantomData,
        }
    }
}

impl<I: TryInitialize<T>, T: ?Sized> TryInitialize<T> for AsPinInit<I, T> {
    type Error = I::Error;

    #[inline]
    fn try_init(self, ptr: Uninit<T>) -> Result<Init<T>, Self::Error> {
        self.init.try_init(ptr)
    }
}

impl<I: TryInitialize<T>, T: ?Sized + Unpin> TryPinInitialize<T> for AsPinInit<I, T> {
    type Error = I::Error;

    #[inline]
    fn try_pin_init(self, ptr: PinnedUninit<T>) -> Result<PinnedInit<T>, Self::Error> {
        match self.init.try_init(ptr.into_inner()) {
            Ok(init) => Ok(PinnedInit::new(init)),
            Err(_) => todo!(),
        }
    }
}

/// a combinator created from [`TryInitialize::to_pin_init`]
pub struct AsInit<I, T: ?Sized> {
    init: I,
    _ty: PhantomData<fn() -> T>,
}

impl<I, T: ?Sized> AsInit<I, T> {
    /// Create a new AsPinInit
    #[inline(always)]
    pub fn new(init: I) -> Self {
        Self {
            init,
            _ty: PhantomData,
        }
    }
}

impl<I: TryPinInitialize<T>, T: ?Sized + Unpin> TryInitialize<T> for AsInit<I, T> {
    type Error = I::Error;

    #[inline]
    fn try_init(self, ptr: Uninit<T>) -> Result<Init<T>, Self::Error> {
        match self.init.try_pin_init(PinnedUninit::new(ptr)) {
            Ok(init) => Ok(PinnedInit::into_inner(init)),
            Err(err) => Err(err),
        }
    }
}

impl<I: TryPinInitialize<T>, T: ?Sized> TryPinInitialize<T> for AsInit<I, T> {
    type Error = I::Error;

    #[inline]
    fn try_pin_init(self, ptr: PinnedUninit<T>) -> Result<PinnedInit<T>, Self::Error> {
        self.init.try_pin_init(ptr)
    }
}
