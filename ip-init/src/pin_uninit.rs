//! The [`PinnedUninit`] type which is analogous to the non-existent [`Pin<Uninit<T>>`]

use crate::{Init, Uninit};

use core::{mem::MaybeUninit, pin::Pin};

mod raw;
pub use raw::PinnedUninit;

impl<T> Default for PinnedUninit<'_, [T]> {
    fn default() -> Self {
        // SAFETY: a zero-sized slice can always be pinned
        unsafe { Self::new_unchecked(Default::default()) }
    }
}

impl<'a, T: ?Sized + Unpin> PinnedUninit<'a, T> {
    /// Analogous to [`Pin::new`]
    pub fn new(uninit: Uninit<'a, T>) -> Self {
        // SAFETY: the value pointed to is `Unpin`, and so has no requirements
        // around pinning.
        unsafe { Self::new_unchecked(uninit) }
    }

    /// Analogous to [`Pin::into_inner`]
    pub fn into_inner(self) -> Uninit<'a, T> {
        // SAFETY: the value pointed to is `Unpin`, and so has no requirements
        // around pinning.
        unsafe { self.into_inner_unchecked() }
    }
}

impl<'a, T: ?Sized> PinnedUninit<'a, T> {
    unsafe fn map_initializer<F: FnOnce(Uninit<'_, T>) -> Init<'_, T>>(
        self,
        f: F,
    ) -> Pin<Init<'a, T>> {
        // SAFETY: the pointee is untouched and the pointer is kept in the pinned type-state
        let uninit = unsafe { self.into_inner_unchecked() };
        // SAFETY: the initialization requirements are forwarded to [`PinnedUninit::assume_init`]
        let init = f(uninit);
        // SAFETY: The `PinnedUninit` guarantees that we are in the pinned type-state now
        // so it's safe to pint the init
        unsafe { Pin::new_unchecked(init) }
    }

    /// Extracts the initialized value from the `Uninit<T>` container.
    /// This is a great way to ensure that the data will get dropped,
    ///  because the resulting `Init<T>` is subject to the usual drop handling.
    ///
    /// # Safety
    ///
    /// the Uninit must have been initialized to a valid instance of T
    pub unsafe fn assume_init(self) -> Pin<Init<'a, T>> {
        // SAFETY: the pointee is untouched and the pointer is kept in the pinned type-state
        //         the initialization requirements are forwarded to [`PinnedUninit::assume_init`]
        unsafe { self.map_initializer(|uninit| uninit.assume_init()) }
    }
}

impl<'a, T> PinnedUninit<'a, T> {
    /// Create an [`Uninit<'_, T>`](Uninit) from a pointer to a `MaybeUninit<T>`
    pub fn from_maybe_uninit(ptr: Pin<&'a mut MaybeUninit<T>>) -> Self {
        // SAFETY: the pointee is untouched and the pointer is kept in the pinned type-state
        let ptr = unsafe { Pin::into_inner_unchecked(ptr) };
        let ptr = Uninit::from_maybe_uninit(ptr);
        // SAFETY: the pointer was pinned before
        unsafe { Self::new_unchecked(ptr) }
    }

    /// Sets the value of the `Uninit<[T]>`
    ///
    /// This overwrites any previous value without dropping it.
    /// This also returns a `Init<'_, T>` to the now safely initialized
    /// contents of self.
    pub fn write(self, value: T) -> Pin<Init<'a, T>> {
        // SAFETY: the pointee is untouched and the pointer is kept in the pinned type-state
        unsafe { self.map_initializer(|uninit| uninit.write(value)) }
    }
}

impl<'a, T> PinnedUninit<'a, [T]> {
    /// Create an [`Uninit<'_, [T]>`](Uninit) from a pointer to a `[MaybeUninit<T>]`
    pub fn from_maybe_uninit_slice(ptr: Pin<&'a mut [MaybeUninit<T>]>) -> Self {
        // SAFETY: the pointee is untouched and the pointer is kept in the pinned type-state
        let ptr = unsafe { Pin::into_inner_unchecked(ptr) };
        let ptr = Uninit::from_maybe_uninit_slice(ptr);
        // SAFETY: the pointer was pinned before
        unsafe { Self::new_unchecked(ptr) }
    }

    /// Sets the value of the `Uninit<[T]>`
    ///
    /// This overwrites any previous value without dropping it.
    /// This also returns a `Init<'_, T>` to the now safely initialized
    /// contents of self.
    ///
    /// # Panics
    ///
    /// If the length of this slice is not equal to T, this method panics
    pub fn write_array<const N: usize>(self, array: [T; N]) -> Pin<Init<'a, [T]>> {
        // SAFETY: the pointee is untouched and the pointer is kept in the pinned type-state
        unsafe { self.map_initializer(|uninit| uninit.write_array(array)) }
    }

    /// Sets the value of the `Uninit<[T]>`
    ///
    /// This overwrites any previous value without dropping it.
    /// This also returns a `Init<'_, T>` to the now safely initialized
    /// contents of self.
    ///
    /// # Panics
    ///
    /// If the length of this slice is not equal to T, this method panics
    pub fn write_slice(self, slice: &[T]) -> Pin<Init<'a, [T]>>
    where
        T: Copy,
    {
        // SAFETY: the pointee is untouched and the pointer is kept in the pinned type-state
        unsafe { self.map_initializer(|uninit| uninit.write_slice(slice)) }
    }

    /// Returns the length of a slice.
    ///
    /// The returned value is the number of **elements**, not the number of bytes.
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.as_ptr().len()
    }

    /// Returns `true` if the slice has a length of 0.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
