//! The [`PinnedUninit`] type which is analogous to the non-existent [`Pin<Uninit<T>>`] and the
//! [`PinnedInit`] type which is analogous to the non-existent [`Pin<Init<T>>`].
//!
//! These are needed because `Uninit` doesn't implement `Deref`
//! and because `Pin` has the extra requirement that the pointee must be dropped before deallocation.
//! This requirement makes it impossible to compose pinned initializers and remain panic/error-safe
//! because it is safe to `mem::forget` a `Pin<Init<T>>` which means it could be deallocated
//! and never dropped.
//!
//! So the `PinnedInit` wrapper type does away with this requirement. However this is a transient
//! requirement, because the owner of the allocation can provide the additional guarntee that
//! drop will be run before deallocation once the type is fully initiialized.
//!
//! So it's still possible to write a correct `emplace_pin(init) -> Pin<Box<T>>`.
//! And stack pin initialization must yield a `Pin<&mut T>` just like normal stack pins.

use crate::{
    traits::{PinInitialize, TryPinInitialize},
    Init, Uninit,
};

use core::{mem::MaybeUninit, ops::Deref, pin::Pin};

mod raw;
pub use raw::{PinnedInit, PinnedUninit};

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

impl<'a, T, const N: usize> PinnedUninit<'a, [T; N]> {
    /// Convert a pointer to an array to a pointer to a slice
    pub fn into_slice(self) -> PinnedUninit<'a, [T]> {
        // SAFETY: we don't touch the pointee, so it stays in the pinned type state
        unsafe { PinnedUninit::new_unchecked(self.into_inner_unchecked().into_slice()) }
    }
}

impl<'a, T: ?Sized> PinnedUninit<'a, T> {
    unsafe fn map_initializer<F: FnOnce(Uninit<'_, T>) -> Init<'_, T>>(
        self,
        f: F,
    ) -> PinnedInit<'a, T> {
        // SAFETY: the pointee is untouched and the pointer is kept in the pinned type-state
        let uninit = unsafe { self.into_inner_unchecked() };
        // SAFETY: the initialization requirements are forwarded to [`PinnedUninit::assume_init`]
        let init = f(uninit);
        // SAFETY: The `PinnedUninit` guarantees that we are in the pinned type-state now
        // so it's safe to pint the init
        unsafe { PinnedInit::new_unchecked(init) }
    }

    /// Extracts the initialized value from the `Uninit<T>` container.
    /// This is a great way to ensure that the data will get dropped,
    ///  because the resulting `Init<T>` is subject to the usual drop handling.
    ///
    /// # Safety
    ///
    /// the Uninit must have been initialized to a valid instance of T
    pub unsafe fn assume_init(self) -> PinnedInit<'a, T> {
        // SAFETY: the pointee is untouched and the pointer is kept in the pinned type-state
        //         the initialization requirements are forwarded to [`PinnedUninit::assume_init`]
        unsafe { self.map_initializer(|uninit| uninit.assume_init()) }
    }

    /// Initialize this pointer
    pub fn try_init<I: TryPinInitialize<T>>(self, init: I) -> Result<PinnedInit<'a, T>, I::Error> {
        init.try_pin_init(self)
    }

    /// Initialize this pointer
    pub fn init<I: PinInitialize<T>>(self, init: I) -> PinnedInit<'a, T> {
        init.pin_init(self)
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
    pub fn write(self, value: T) -> PinnedInit<'a, T> {
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
    pub fn write_array<const N: usize>(self, array: [T; N]) -> PinnedInit<'a, [T]> {
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
    pub fn write_slice(self, slice: &[T]) -> PinnedInit<'a, [T]>
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

impl<T: ?Sized> Deref for PinnedInit<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: this pointer is well aligned and initalized
        // because this is guarnateed by the `Init` that `PinnedInit` wraps
        unsafe { &*self.as_ptr() }
    }
}

impl<'a, T: ?Sized + Unpin> PinnedInit<'a, T> {
    /// Create a new `PinnedInit`
    pub fn new(ptr: Init<'a, T>) -> Self {
        // SAFETY: `T: Unpin` so it doesn't care about pinning
        unsafe { Self::new_unchecked(ptr) }
    }

    /// Unwrap a `PinnedInit` into a `Init`
    pub fn into_inner(this: Self) -> Init<'a, T> {
        // SAFETY: `T: Unpin` so it doesn't care about pinning
        unsafe { Self::into_inner_unchecked(this) }
    }
}

impl<'a, T, const N: usize> TryFrom<PinnedInit<'a, [T]>> for PinnedInit<'a, [T; N]> {
    type Error = PinnedInit<'a, [T]>;

    fn try_from(value: PinnedInit<'a, [T]>) -> Result<Self, Self::Error> {
        if value.len() == N {
            // SAFETY: the ptr is
            // * allocated for T's layout
            // * writable for T's layout
            // * readable for T's layout after written
            // because it's coming from an `Uninit`
            // the pointee isn't touched, so it remains in the pinned type-state
            Ok(unsafe {
                Self::new_unchecked(Init::from_raw_nonnull(
                    PinnedInit::into_inner_unchecked(value).into_raw().cast(),
                ))
            })
        } else {
            Err(value)
        }
    }
}
