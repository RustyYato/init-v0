//! slice initializers and writers
//!
//! this allows you to safely initialize the entire uninitialized slice efficiently,
//! and drop initialized elements on error.

use crate::traits::{TryInitialize, TryPinInitialize};

/// An array initializer
pub struct ArrayInit<I>(I);

impl<I> ArrayInit<I> {
    /// Create a new array initializer
    pub fn new(init: I) -> Self {
        Self(init)
    }
}

impl<I: TryInitialize<[T]>, T, const N: usize> TryInitialize<[T; N]> for ArrayInit<I> {
    type Error = I::Error;

    fn try_init(self, ptr: crate::Uninit<[T; N]>) -> Result<crate::Init<[T; N]>, Self::Error> {
        match self.0.try_init(ptr.into_slice()) {
            // SAFETY: this init is guarnteed to be the same pointer as the `ptr` passed into `self.0.try_init`
            // So it has the correct length
            Ok(init) => Ok(unsafe { init.try_into().unwrap_unchecked() }),
            Err(err) => Err(err),
        }
    }
}

impl<I: TryPinInitialize<[T]>, T, const N: usize> TryPinInitialize<[T; N]> for ArrayInit<I> {
    type Error = I::Error;

    fn try_pin_init(
        self,
        ptr: crate::PinnedUninit<[T; N]>,
    ) -> Result<crate::PinnedInit<[T; N]>, Self::Error> {
        match self.0.try_pin_init(ptr.into_slice()) {
            // SAFETY: this init is guarnteed to be the same pointer as the `ptr` passed into `self.0.try_init`
            // So it has the correct length
            Ok(init) => Ok(unsafe { init.try_into().unwrap_unchecked() }),
            Err(err) => Err(err),
        }
    }
}
