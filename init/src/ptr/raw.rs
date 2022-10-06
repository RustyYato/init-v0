use core::{marker::PhantomData, ptr::NonNull};

struct Invariant<'a>(&'a mut &'a mut ());

/// A pointer to uninitialized but allocated memory
pub struct Uninit<'a, T: ?Sized> {
    ptr: NonNull<T>,
    lt: PhantomData<(&'a T, Invariant<'a>)>,
}

impl<'a, T: ?Sized> Uninit<'a, T> {
    /// # Safety
    ///
    /// The pointer must be
    /// * non-null
    /// * allocated for T's layout
    /// * writable for T's layout
    /// * readable for T's layout after written to
    #[inline(always)]
    pub unsafe fn from_raw(ptr: *mut T) -> Self {
        // SAFETY: the pointer is non-null
        // the safety requirements of `from_raw_nonnull` are forwarded to `from_raw`
        unsafe { Self::from_raw_nonnull(NonNull::new_unchecked(ptr)) }
    }

    /// # Safety
    ///
    /// The pointer must be
    /// * allocated for T's layout
    /// * writable for T's layout
    /// * readable for T's layout after written to
    #[inline(always)]
    pub unsafe fn from_raw_nonnull(ptr: NonNull<T>) -> Self {
        Self {
            ptr,
            lt: PhantomData,
        }
    }

    /// Acquires the underlying pointer.
    ///
    /// This pointer is not valid for writes
    pub fn as_ptr(&self) -> *const T {
        self.ptr.as_ptr()
    }

    /// Acquires the underlying pointer.
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr.as_ptr()
    }

    /// Acquires the underlying pointer.
    pub fn as_non_null_ptr(&self) -> NonNull<T> {
        self.ptr
    }
}

/// A pointer to initialized memory, and proof initialization
///
/// For example:
/// if you have the following function, if it returns the `Init<u8>` then it is guaranteed that the
/// pointer passed in was initialized.
///
/// NOTE: if the function panics or is fallible and doesn't return a `Init<u8>`, then nothing about
/// the state is guaranteed by this library. The function may give more guarantees.
///
/// ```
/// use init::{Uninit, Init};
/// fn init_me(ptr: Uninit<u8>) -> Init<u8> {
///     // ...
/// }
/// ```
pub struct Init<'a, T: ?Sized> {
    ptr: NonNull<T>,
    lt: PhantomData<(&'a T, Invariant<'a>, T)>,
}

// SAFETY: The Init contains a T and allows accessing a T by &mut T
unsafe impl<'a, T: Send> Send for Init<'a, T> {}
// SAFETY: The Init contains a T and allows accessing a T by &T
unsafe impl<'a, T: Sync> Sync for Init<'a, T> {}

impl<'a, T: ?Sized> Init<'a, T> {
    /// # Safety
    ///
    /// The pointer must be
    /// * non-null
    /// * allocated for T's layout
    /// * writable for T's layout
    /// * readable for T's layout
    /// The pointee must be a valid instance of T
    #[inline(always)]
    pub unsafe fn from_raw(ptr: *mut T) -> Self {
        // SAFETY: the pointer is non-null
        // the safety requirements of `from_raw_nonnull` are forwarded to `from_raw`
        unsafe { Self::from_raw_nonnull(NonNull::new_unchecked(ptr)) }
    }

    /// # Safety
    ///
    /// The pointer must be
    /// * allocated for T's layout
    /// * writable for T's layout
    /// * readable for T's layout
    /// The pointee must be a valid instance of T
    #[inline(always)]
    pub unsafe fn from_raw_nonnull(ptr: NonNull<T>) -> Self {
        Self {
            ptr,
            lt: PhantomData,
        }
    }

    /// Acquires the underlying pointer.
    ///
    /// This pointer is not valid for writes
    #[inline(always)]
    pub fn as_ptr(&self) -> *const T {
        self.ptr.as_ptr()
    }

    /// Acquires the underlying pointer.
    #[inline(always)]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr.as_ptr()
    }

    /// Acquires the underlying pointer.
    ///
    /// This does *NOT* drop self
    #[inline(always)]
    pub fn into_raw(self) -> NonNull<T> {
        let ptr = self.ptr;
        core::mem::forget(self);
        ptr
    }

    /// Acquires the underlying pointer.
    #[inline(always)]
    pub fn as_non_null_ptr(&self) -> NonNull<T> {
        self.ptr
    }
}

// SAFETY: the drop impl only drops the T, so is trivially correct for #[may_dangle]
unsafe impl<#[may_dangle] T: ?Sized> Drop for Init<'_, T> {
    fn drop(&mut self) {
        // SAFETY: The pointee is a valid instance of T
        unsafe { self.as_mut_ptr().drop_in_place() }
    }
}
