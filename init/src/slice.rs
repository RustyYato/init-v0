//! slice initializers

mod pin_writer;
mod writer;
use core::pin::Pin;

pub use pin_writer::PinSliceWriter;
pub use writer::SliceWriter;

use crate::{TryInitialize, TryPinInitialize};

/// Create a new slice initializer
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
