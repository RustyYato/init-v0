#[cfg(doc)]
use core::pin::Pin;
use core::ptr::NonNull;

use crate::Uninit;

/// Analogous to `Pin<Uninit<_>>` which can't exist because `Uninit` doesn't implement `Deref`
#[repr(transparent)]
pub struct PinnedUninit<'a, T: ?Sized> {
    uninit: Uninit<'a, T>,
}

impl<'a, T: ?Sized> PinnedUninit<'a, T> {
    /// Analogous to [`Pin::new_unchecked`]
    ///
    /// Construct a new `PinnedUninit<T>` around a reference to some data of a type that may or may not implement `Unpin`.
    ///
    /// If pointer dereferences to an `Unpin` type, [`PinnedUninit::new`] should be used instead.
    ///
    /// # Safety
    ///
    /// This constructor is unsafe because we cannot guarantee that the data pointed to by pointer
    /// is pinned, meaning that the data will not be moved or its storage invalidated until it
    /// gets dropped. If the constructed Pin<P> does not guarantee that the data P points to is
    /// pinned, that is a violation of the API contract and may lead to undefined behavior in
    /// later (safe) operations.
    ///
    /// For example, calling Pin::new_unchecked on an &'a mut T is unsafe because while you are
    /// able to pin it for the given lifetime 'a, you have no control over whether it is kept
    /// pinned once 'a ends:
    pub unsafe fn new_unchecked(uninit: Uninit<'a, T>) -> Self {
        Self { uninit }
    }

    /// Analogous to [`Pin::into_inner_unchecked`]
    ///
    /// # Safety
    ///
    /// This function is unsafe. You must guarantee that you will continue to treat the
    /// [`Uninit<T>`] as pinned after you call this function, so that the invariants on
    /// the [`PinnedUninit<T>`] type can be upheld. If the code using the resulting [`Uninit<T>`]
    ///  does not continue to maintain the pinning invariants that is a violation of
    ///  the API contract and may lead to undefined behavior in later (safe) operations.
    ///
    /// If the underlying data is `Unpin`, [`Pin::into_inner`] should be used instead.
    pub unsafe fn into_inner_unchecked(self) -> Uninit<'a, T> {
        self.uninit
    }

    /// Acquires the underlying pointer.
    ///
    /// This pointer is not valid for writes
    pub fn as_ptr(&self) -> *const T {
        self.uninit.as_ptr()
    }

    /// Acquires the underlying pointer.
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.uninit.as_mut_ptr()
    }

    /// Acquires the underlying pointer.
    pub fn as_non_null_ptr(&self) -> NonNull<T> {
        self.uninit.as_non_null_ptr()
    }
}
