use core::{marker::PhantomData, pin::Pin};

use crate::{
    traits::{PinInitialize, TryPinInitialize},
    Init, PinnedUninit,
};

/// A writer to a pinned uninitialized slice
pub struct PinSliceWriter<'a, T> {
    uninit: PinnedUninit<'a, [T]>,
    current: *mut T,
    remaining: usize,
    _lt: PhantomData<PinnedUninit<'a, T>>,
}

// SAFETY: this only drops the T, so is trivially correct for `#[may_dangle]`
unsafe impl<#[may_dangle] T> Drop for PinSliceWriter<'_, T> {
    fn drop(&mut self) {
        let len = self.uninit.len();
        let ptr = self.uninit.as_mut_ptr().cast::<T>();
        let len = len.wrapping_sub(self.remaining);
        let ptr = core::ptr::slice_from_raw_parts_mut(ptr, len);
        // SAFETY: this only drops the initialized portion of the writer
        unsafe { ptr.drop_in_place() }
    }
}

impl<'a, T> PinSliceWriter<'a, T> {
    /// create a new writer
    pub fn new(mut uninit: PinnedUninit<'a, [T]>) -> Self {
        let len = uninit.len();
        let ptr = uninit.as_mut_ptr().cast::<T>();
        Self {
            uninit,
            current: ptr,
            remaining: len,
            _lt: PhantomData,
        }
    }

    /// Try to apply the function to all remaining unintialized slots in the slice
    /// and return the fully initialized slice, unless the function fails.
    /// In which case return the error.
    pub fn try_for_each<E>(
        mut self,
        mut f: impl FnMut(PinnedUninit<'_, T>) -> Result<Pin<Init<'_, T>>, E>,
    ) -> Result<Pin<Init<'a, [T]>>, E> {
        while !self.is_finished() {
            self.try_write(crate::func::TryPinInitFn::new(&mut f))?
        }

        Ok(self.finish())
    }

    /// Apply the function to all remaining unintialized slots in the slice
    /// and return the fully initialized slice
    pub fn for_each<E>(
        mut self,
        mut f: impl FnMut(PinnedUninit<'_, T>) -> Pin<Init<'_, T>>,
    ) -> Pin<Init<'a, [T]>> {
        while !self.is_finished() {
            self.try_write(crate::func::PinInitFn::new(&mut f))
                .unwrap_or_else(|inf| match inf {})
        }

        self.finish()
    }

    /// finish the writer and get an initialized slice
    ///
    /// # Panics
    ///
    /// if the writer isn't finished, this function will panic
    #[inline]
    pub fn finish(self) -> Pin<Init<'a, [T]>> {
        assert!(self.is_finished());
        // SAFETY: this writer is finished
        unsafe { self.finish_unchecked() }
    }

    /// finish the writer and get an initialized slice
    ///
    /// # Safety
    ///
    /// if the writer must be finished
    #[inline]
    pub unsafe fn finish_unchecked(mut self) -> Pin<Init<'a, [T]>> {
        let uninit = core::mem::take(&mut self.uninit);
        core::mem::forget(self);
        // SAFETY: a finished writer has initialized every element of the slice
        unsafe { uninit.assume_init() }
    }

    /// Has the writer written to the entire slice
    #[inline(always)]
    pub fn is_finished(&self) -> bool {
        self.remaining == 0
    }

    /// Try to initialize the next slot
    ///
    /// # Panics
    ///
    /// if the writer is finished, this function will panic
    pub fn try_write<I: TryPinInitialize<T>>(&mut self, init: I) -> Result<(), I::Error> {
        assert!(!self.is_finished());
        // SAFETY: we're not finished yet
        unsafe { self.try_init_unchecked(init) }
    }

    /// Try to initialize the next slot
    ///
    /// # Safety
    ///
    /// The writer must not be finished yet
    pub unsafe fn try_init_unchecked<I: TryPinInitialize<T>>(
        &mut self,
        init: I,
    ) -> Result<(), I::Error> {
        debug_assert!(!self.is_finished());

        // SAFETY:
        // * the current pointer came from an uninit
        // * the writer isn't finished yet
        // therefore the pointer is still in bounds
        let output = unsafe { crate::raw::try_pin_init_in_place(init, self.current) };

        if output.is_ok() {
            // SAFETY: we aren't finished yet and the current slot was successfully initialized
            self.current = unsafe { self.current.add(1) };
            self.remaining -= 1;
        }

        output
    }

    /// Try to initialize the next slot
    ///
    /// # Panics
    ///
    /// if the writer is finished, this function will panic
    pub fn init<I: PinInitialize<T>>(&mut self, init: I) {
        assert!(!self.is_finished());
        // SAFETY: we're not finished yet
        unsafe { self.init_unchecked(init) }
    }

    /// Try to initialize the next slot
    ///
    /// # Safety
    ///
    /// The writer must not be finished yet
    pub unsafe fn init_unchecked<I: PinInitialize<T>>(&mut self, init: I) {
        debug_assert!(!self.is_finished());

        // SAFETY:
        // * the current pointer came from an uninit
        // * the writer isn't finished yet
        // therefore the pointer is still in bounds
        unsafe { crate::raw::pin_init_in_place(init, self.current) }

        // SAFETY: we aren't finished yet and the current slot was successfully initialized
        self.current = unsafe { self.current.add(1) };
        self.remaining -= 1;
    }
}
