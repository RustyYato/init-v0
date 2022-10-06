mod raw;

use core::ptr;
use core::{mem::MaybeUninit, ptr::NonNull};

pub use raw::{Init, Uninit};

impl<'a, T: ?Sized> Uninit<'a, T> {
    /// Extracts the initialized value from the `Uninit<T>` container.
    /// This is a great way to ensure that the data will get dropped,
    ///  because the resulting `Init<T>` is subject to the usual drop handling.
    ///
    /// # Safety
    ///
    /// the Uninit must have been initialized to a valid instance of T
    pub unsafe fn assume_init(self) -> Init<'a, T> {
        // SAFETY:
        //
        // The pointer is (because `Self = Uninit<T>`)
        // * allocated for T's layout
        // * writable for T's layout
        // * readable for T's layout
        //
        // The pointee is a valid instance of T because of safety
        // condition on `assume_init`
        unsafe { Init::from_raw_nonnull(self.as_non_null_ptr()) }
    }
}

impl<'a, T> Uninit<'a, T> {
    /// Create an [`Uninit<'_, T>`](Uninit) from a pointer to a `MaybeUninit<T>`
    pub fn from_maybe_uninit(ptr: &'a mut MaybeUninit<T>) -> Self {
        let ptr = NonNull::from(ptr);
        let ptr = ptr.cast::<T>();
        // SAFETY: the ptr is
        // * allocated for T's layout
        // * writable for T's layout
        // * readable for T's layout after written to
        unsafe { Self::from_raw_nonnull(ptr) }
    }

    /// Sets the value of the `Uninit<[T]>`
    ///
    /// This overwrites any previous value without dropping it.
    /// This also returns a `Init<'_, T>` to the now safely initialized
    /// contents of self.
    pub fn write(mut self, value: T) -> Init<'a, T> {
        // SAFETY: the pointer is guaranteed to be valid for writes
        unsafe { self.as_mut_ptr().write(value) }
        // SAFETY: the slice was initialized by the write above
        unsafe { self.assume_init() }
    }
}

impl<'a, T> Uninit<'a, [T]> {
    /// Create an [`Uninit<'_, [T]>`](Uninit) from a pointer to a `[MaybeUninit<T>]`
    pub fn from_maybe_uninit_slice(ptr: &'a mut [MaybeUninit<T>]) -> Self {
        let ptr = ptr::slice_from_raw_parts_mut(ptr.as_mut_ptr().cast::<T>(), ptr.len());
        // SAFETY: the pointer is non-null
        let ptr = unsafe { NonNull::new_unchecked(ptr) };
        // SAFETY: the ptr is
        // * allocated for T's layout
        // * writable for T's layout
        // * readable for T's layout after written to
        unsafe { Self::from_raw_nonnull(ptr) }
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
    pub fn write_array<const N: usize>(mut self, array: [T; N]) -> Init<'a, [T]> {
        assert!(self.len() == N);
        // SAFETY: [T] has the same layout as [T; N] where [T]::len == N
        unsafe { self.as_mut_ptr().cast::<[T; N]>().write(array) }
        // SAFETY: the slice was initialized by the write above
        unsafe { self.assume_init() }
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
    pub fn write_slice(mut self, slice: &[T]) -> Init<'a, [T]>
    where
        T: Copy,
    {
        assert!(self.len() == slice.len());
        // SAFETY: the two slices have the same length
        unsafe {
            self.as_mut_ptr()
                .cast::<T>()
                .copy_from_nonoverlapping(slice.as_ptr(), self.len())
        }
        // SAFETY: the slice was initialized by the write above
        unsafe { self.assume_init() }
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

impl<T: ?Sized> core::ops::Deref for Init<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: the pointe is a valid instance of T
        unsafe { &*self.as_ptr() }
    }
}

impl<T: ?Sized> core::ops::DerefMut for Init<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: the pointe is a valid instance of T
        // the pointer is valid for writes
        unsafe { &mut *self.as_mut_ptr() }
    }
}
