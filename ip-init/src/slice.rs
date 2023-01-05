//! slice initializers and writers
//!
//! this allows you to safely initialize the entire uninitialized slice efficiently,
//! and drop initialized elements on error.

mod pin_writer;
mod writer;

pub use pin_writer::PinSliceWriter;
pub use writer::SliceWriter;

use crate::traits::{TryInitialize, TryPinInitialize};

/// A slice initializer which clones the provided initializer to initialize each element
pub struct SliceInit<I>(I);

impl<I> SliceInit<I> {
    /// Create a new slice initializer
    pub fn new(init: I) -> Self {
        Self(init)
    }
}

impl<I: TryInitialize<T> + Clone, T> TryInitialize<[T]> for SliceInit<I> {
    type Error = I::Error;

    fn try_init(self, ptr: crate::Uninit<[T]>) -> Result<crate::Init<[T]>, Self::Error> {
        SliceWriter::new(ptr).try_for_each(|uninit| uninit.try_init(self.0.clone()))
    }
}

impl<I: TryPinInitialize<T> + Clone, T> TryPinInitialize<[T]> for SliceInit<I> {
    type Error = I::Error;

    fn try_pin_init(
        self,
        ptr: crate::PinnedUninit<[T]>,
    ) -> Result<crate::PinnedInit<[T]>, Self::Error> {
        PinSliceWriter::new(ptr).try_for_each(|uninit| uninit.try_init(self.0.clone()))
    }
}

impl<I: TryInitialize<T> + Clone, T, const N: usize> TryInitialize<[T; N]> for SliceInit<I> {
    type Error = I::Error;

    fn try_init(self, ptr: crate::Uninit<[T; N]>) -> Result<crate::Init<[T; N]>, Self::Error> {
        super::array::ArrayInit::new(self).try_init(ptr)
    }
}

impl<I: TryPinInitialize<T> + Clone, T, const N: usize> TryPinInitialize<[T; N]> for SliceInit<I> {
    type Error = I::Error;

    fn try_pin_init(
        self,
        ptr: crate::PinnedUninit<[T; N]>,
    ) -> Result<crate::PinnedInit<[T; N]>, Self::Error> {
        super::array::ArrayInit::new(self).try_pin_init(ptr)
    }
}

/// A slice initializer which clones the provided initializer to initialize each element
pub struct SliceIterInit<I>(I);

impl<I: Iterator> SliceIterInit<I> {
    /// Create a new slice initializer
    pub fn new(iter: I) -> Self {
        Self(iter)
    }
}

/// The Error type of `SliceIterInit`
pub enum SliceIterInitError<T> {
    /// If the underlying iterator didn't yield enough items
    NotEnoughItems,
    /// If the initializer produced by the iterator errored
    Init(T),
}

impl<I: Iterator, T> TryInitialize<[T]> for SliceIterInit<I>
where
    I::Item: TryInitialize<T>,
{
    type Error = SliceIterInitError<<I::Item as TryInitialize<T>>::Error>;

    fn try_init(mut self, ptr: crate::Uninit<[T]>) -> Result<crate::Init<[T]>, Self::Error> {
        SliceWriter::new(ptr).try_for_each(|uninit| {
            uninit
                .try_init(self.0.next().ok_or(SliceIterInitError::NotEnoughItems)?)
                .map_err(SliceIterInitError::Init)
        })
    }
}

impl<I: Iterator, T> TryPinInitialize<[T]> for SliceIterInit<I>
where
    I::Item: TryPinInitialize<T>,
{
    type Error = SliceIterInitError<<I::Item as TryPinInitialize<T>>::Error>;

    fn try_pin_init(
        mut self,
        ptr: crate::PinnedUninit<[T]>,
    ) -> Result<crate::PinnedInit<[T]>, Self::Error> {
        PinSliceWriter::new(ptr).try_for_each(|uninit| {
            uninit
                .try_init(self.0.next().ok_or(SliceIterInitError::NotEnoughItems)?)
                .map_err(SliceIterInitError::Init)
        })
    }
}

impl<I: Iterator, T, const N: usize> TryInitialize<[T; N]> for SliceIterInit<I>
where
    I::Item: TryInitialize<T>,
{
    type Error = SliceIterInitError<<I::Item as TryInitialize<T>>::Error>;

    fn try_init(self, ptr: crate::Uninit<[T; N]>) -> Result<crate::Init<[T; N]>, Self::Error> {
        super::array::ArrayInit::new(self).try_init(ptr)
    }
}

impl<I: Iterator, T, const N: usize> TryPinInitialize<[T; N]> for SliceIterInit<I>
where
    I::Item: TryPinInitialize<T>,
{
    type Error = SliceIterInitError<<I::Item as TryPinInitialize<T>>::Error>;

    fn try_pin_init(
        self,
        ptr: crate::PinnedUninit<[T; N]>,
    ) -> Result<crate::PinnedInit<[T; N]>, Self::Error> {
        super::array::ArrayInit::new(self).try_pin_init(ptr)
    }
}
