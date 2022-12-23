//! slice initializers and writers
//!
//! this allows you to safely initialize the entire uninitialized slice efficiently,
//! and drop initialized elements on error.

mod pin_writer;
mod writer;
use core::pin::Pin;

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
        let mut writer = SliceWriter::new(ptr);

        while !writer.is_finished() {
            writer.try_init(self.0.clone())?
        }

        Ok(writer.finish())
    }
}

impl<I: TryPinInitialize<T> + Clone, T> TryPinInitialize<[T]> for SliceInit<I> {
    type Error = I::Error;

    fn try_pin_init(
        self,
        ptr: crate::PinnedUninit<[T]>,
    ) -> Result<Pin<crate::Init<[T]>>, Self::Error> {
        let mut writer = PinSliceWriter::new(ptr);

        while !writer.is_finished() {
            writer.try_init(self.0.clone())?
        }

        Ok(writer.finish())
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
        let mut writer = SliceWriter::new(ptr);

        while !writer.is_finished() {
            writer
                .try_init(self.0.next().ok_or(SliceIterInitError::NotEnoughItems)?)
                .map_err(SliceIterInitError::Init)?
        }

        Ok(writer.finish())
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
    ) -> Result<Pin<crate::Init<[T]>>, Self::Error> {
        let mut writer = PinSliceWriter::new(ptr);

        while !writer.is_finished() {
            writer
                .try_init(self.0.next().ok_or(SliceIterInitError::NotEnoughItems)?)
                .map_err(SliceIterInitError::Init)?
        }

        Ok(writer.finish())
    }
}
