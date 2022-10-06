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
pub use ptr::{Init, Uninit};

/// A trait to initialize a T
pub trait TryInitialize<T: ?Sized> {
    /// the error reported by this
    type Error;

    /// attempt to initialize the pointer
    ///
    /// if this function returns Ok, then the ptr was initialized
    /// if this function returns Err, then the ptr may not be initialized
    fn try_init(self, ptr: Uninit<T>) -> Result<Init<T>, Self::Error>;
}

/// A trait to initialize a T
pub trait Initialize<T: ?Sized>: TryInitialize<T, Error = core::convert::Infallible> {
    /// initializes the ptr
    fn init(self, ptr: Uninit<T>) -> Init<T>;
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
