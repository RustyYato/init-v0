//! the fundamental traits that underpin this crate

use core::{
    alloc::{Layout, LayoutError},
    ptr::NonNull,
};

use crate::{
    pin::{AsInit, AsPinInit},
    pin_ptr::{PinnedInit, PinnedUninit},
    slice::SliceInit,
    Init, Uninit,
};

/// A layout provider takes a pair of an initializer and a type, and provides the layout that should be used for the type
///
/// # Safety
///
/// * The layout of `T` must fit the allocation provided by `layout_for`
/// * The `cast` function must return the same pointer that it was provided
pub unsafe trait LayoutProvider<T: ?Sized> {
    /// The layout of T, given the initializer
    fn layout_for(&self) -> Result<Layout, LayoutError>;

    /// Casts the pointer to `T`
    fn cast(&self, ptr: *mut u8) -> *mut T;

    /// Casts the pointer to `T`
    fn cast_nonnull(&self, ptr: NonNull<u8>) -> NonNull<T> {
        // SAFETY: the pointer is guaranteed to be non-null because
        // Self::cast will return the same pointer that it was passed in
        // and ptr is non-null
        unsafe { NonNull::new_unchecked(self.cast(ptr.as_ptr())) }
    }
}

/// A trait to try to initialize a T
///
/// * for infallible initialization, use [`Initialize`]
/// * for pinned initialization, use [`TryPinInitialize`]
pub trait TryInitialize<T: ?Sized> {
    /// the error reported by this
    type Error;

    /// attempt to initialize the pointer
    ///
    /// if this function returns [`Ok`], then the ptr was initialized
    /// otherwise, then the ptr may not be initialized
    fn try_init(self, ptr: Uninit<T>) -> Result<Init<T>, Self::Error>;

    /// Convert this [`TryInitialize`] to a [`TryPinInitialize`]
    fn to_pin_init(self) -> AsPinInit<Self, T>
    where
        Self: Sized,
        T: Unpin,
    {
        AsPinInit::new(self)
    }

    /// Convert this initializer to a slice initializer
    fn to_slice_init(self) -> SliceInit<Self>
    where
        Self: Clone,
        T: Sized,
    {
        SliceInit::new(self)
    }
}

/// A trait to initialize a T
///
/// * for infallible initialization, use [`PinInitialize`]
/// * for normal initialization, use [`TryInitialize`]
pub trait TryPinInitialize<T: ?Sized> {
    /// the error reported by this
    type Error;

    /// attempt to initialize the pointer
    ///
    /// if this function returns [`Ok`], then the ptr was initialized
    /// otherwise, then the ptr may not be initialized
    fn try_pin_init(self, ptr: PinnedUninit<T>) -> Result<PinnedInit<T>, Self::Error>;

    /// Convert this [`TryPinInitialize`] to a [`TryInitialize`]
    fn to_init(self) -> AsInit<Self, T>
    where
        Self: Sized,
        T: Unpin,
    {
        AsInit::new(self)
    }

    /// Convert this initializer to a slice initializer
    fn to_slice_init(self) -> SliceInit<Self>
    where
        Self: Clone,
        T: Sized,
    {
        SliceInit::new(self)
    }
}

/// A trait to initialize a T, this trait is automatically implemented for
/// all valid instances of [`TryInitialize`]
pub trait Initialize<T: ?Sized>: TryInitialize<T, Error = core::convert::Infallible> {
    /// initializes the ptr
    fn init(self, ptr: Uninit<T>) -> Init<T>;
}

/// A trait to pin initialize a T, this trait is automatically implemented for
/// all valid instances of [`TryPinInitialize`]
pub trait PinInitialize<T: ?Sized>: TryPinInitialize<T, Error = core::convert::Infallible> {
    /// attempt to initialize the pointer
    ///
    /// if this function returns Ok, then the ptr was initialized
    /// otherwise, then the ptr may not be initialized
    fn pin_init(self, ptr: PinnedUninit<T>) -> PinnedInit<T>;
}

impl<T> TryInitialize<T> for T {
    type Error = core::convert::Infallible;

    #[inline]
    fn try_init(self, ptr: Uninit<T>) -> Result<Init<T>, Self::Error> {
        Ok(ptr.write(self))
    }
}

impl<T> TryPinInitialize<T> for T {
    type Error = core::convert::Infallible;

    #[inline]
    fn try_pin_init(
        self,
        ptr: crate::PinnedUninit<T>,
    ) -> core::result::Result<PinnedInit<'_, T>, core::convert::Infallible> {
        Ok(ptr.write(self))
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

impl<T: ?Sized, I: TryPinInitialize<T, Error = core::convert::Infallible>> PinInitialize<T> for I {
    fn pin_init(self, ptr: PinnedUninit<T>) -> PinnedInit<T> {
        match self.try_pin_init(ptr) {
            Ok(init) => init,
            Err(err) => match err {},
        }
    }
}
