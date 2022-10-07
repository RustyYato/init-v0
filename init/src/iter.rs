//! slice iterators for [`Uninit`], [`PinnedUninit`], and [`Init`]

use core::{marker::PhantomData, mem, pin::Pin, ptr::NonNull};

use crate::{pin_uninit::PinnedUninit, Init, Uninit};

struct RawIter<T> {
    start: NonNull<T>,
    end: *const T,
}

impl<T> RawIter<T> {
    const ZERO_SIZED: bool = mem::size_of::<T>() == 0;

    unsafe fn new(ptr: NonNull<[T]>) -> Self {
        if Self::ZERO_SIZED {
            Self {
                start: NonNull::dangling(),
                end: ptr.len() as *const T,
            }
        } else {
            let len = ptr.len();
            let ptr = ptr.cast();
            Self {
                start: ptr,
                // SAFETY: the caller of new must pass in an allocated slice
                // and for allocated slices the start+len pointer is still in the allocation
                end: unsafe { ptr.as_ptr().add(len) },
            }
        }
    }

    fn as_slice(&self) -> NonNull<[T]> {
        let len = self.len();

        let ptr = core::ptr::slice_from_raw_parts_mut(self.start.as_ptr(), len);

        // SAFETY: self.start is non-null
        unsafe { NonNull::new_unchecked(ptr) }
    }
}

impl<T> ExactSizeIterator for RawIter<T> {}
impl<T> Iterator for RawIter<T> {
    type Item = NonNull<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if Self::ZERO_SIZED {
            let end = self.end as usize;
            let end = end.checked_sub(1)?;
            self.end = end as _;
            Some(self.start)
        } else if self.end == self.start.as_ptr() {
            None
        } else {
            let ptr = self.start;

            // SAFETY: the pointer still hasn't reached the end, so adding one is safe
            let next = unsafe { self.start.as_ptr().add(1) };
            // SAFETY: the pointer is still in the same allocation, so must be non-null
            self.start = unsafe { NonNull::new_unchecked(next) };

            Some(ptr)
        }
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if Self::ZERO_SIZED {
            let end = self.end as usize;
            let end = end.checked_sub(n)?;
            self.end = end as _;
            self.next()
        } else if self.len() >= n {
            self.end = self.start.as_ptr();
            None
        } else {
            // SAFETY: the pointer still hasn't reached the end, so adding one is safe
            let next = unsafe { self.start.as_ptr().add(n) };
            // SAFETY: the pointer is still in the same allocation, so must be non-null
            let ptr = unsafe { NonNull::new_unchecked(next) };

            // SAFETY: the pointer is still in the same allocation, so must be non-null
            self.start = unsafe { NonNull::new_unchecked(next.add(1)) };

            Some(ptr)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = if Self::ZERO_SIZED {
            self.end as usize
        } else {
            // SAFETY: start and end are in the same allocation
            unsafe { self.end.offset_from(self.start.as_ptr()) as usize }
        };

        (len, Some(len))
    }

    fn count(self) -> usize {
        self.len()
    }

    fn last(mut self) -> Option<Self::Item> {
        self.next_back()
    }
}

impl<T> DoubleEndedIterator for RawIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if Self::ZERO_SIZED {
            self.next()
        } else if self.end == self.start.as_ptr() {
            None
        } else {
            // SAFETY: The end is still in the allocation because it wasn't equal to start
            unsafe { self.end = self.end.sub(1) }

            // SAFETY: the end pointer is always non-null for non-zero sized T
            Some(unsafe { NonNull::new_unchecked(self.end as _) })
        }
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        if Self::ZERO_SIZED {
            self.nth(n)
        } else if self.len() >= n {
            self.end = self.start.as_ptr();
            None
        } else {
            // SAFETY: The end is still in the allocation because it wasn't equal to start
            unsafe { self.end = self.end.sub(n + 1) }

            // SAFETY: the end pointer is always non-null for non-zero sized T
            Some(unsafe { NonNull::new_unchecked(self.end as _) })
        }
    }
}

/// An iterator over uninit pointers
pub struct UninitIter<'a, T> {
    raw: RawIter<T>,
    _lt: PhantomData<Uninit<'a, [T]>>,
}

impl<'a, T> UninitIter<'a, T> {
    /// Create a new iterator over uninit pointers
    pub fn new(uninit: Uninit<'a, [T]>) -> Self {
        Self {
            // SAFETY: the uninit is guaranteed to live for at least as long as 'a
            raw: unsafe { RawIter::new(uninit.as_non_null_ptr()) },
            _lt: PhantomData,
        }
    }

    /// Get the rest of the slice
    pub fn finish(self) -> Uninit<'a, [T]> {
        let ptr = self.raw.as_slice();
        // SAFETY: the slice came from an Uninit so is allocated
        // and the slice doesn't alias with any pointer given out by
        // the iterator
        unsafe { Uninit::from_raw_nonnull(ptr) }
    }
}

impl<'a, T> ExactSizeIterator for UninitIter<'a, T> {}
impl<'a, T> Iterator for UninitIter<'a, T> {
    type Item = Uninit<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let ptr = self.raw.next();
        // SAFETY: this pointer is derived from an uninit pointer
        ptr.map(|ptr| unsafe { Uninit::from_raw_nonnull(ptr) })
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let ptr = self.raw.nth(n);
        // SAFETY: this pointer is derived from an uninit pointer
        ptr.map(|ptr| unsafe { Uninit::from_raw_nonnull(ptr) })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.raw.size_hint()
    }
}
impl<'a, T> DoubleEndedIterator for UninitIter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let ptr = self.raw.next_back();
        // SAFETY: this pointer is derived from an uninit pointer
        ptr.map(|ptr| unsafe { Uninit::from_raw_nonnull(ptr) })
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let ptr = self.raw.nth_back(n);
        // SAFETY: this pointer is derived from an uninit pointer
        ptr.map(|ptr| unsafe { Uninit::from_raw_nonnull(ptr) })
    }
}

/// An iterator over Init pointers
pub struct InitIter<'a, T> {
    raw: RawIter<T>,
    _lt: PhantomData<Init<'a, [T]>>,
}

// SAFETY: this drop impl only drops Ts so is trivially correct for #[may_dangle]
unsafe impl<#[may_dangle] T> Drop for InitIter<'_, T> {
    fn drop(&mut self) {
        // SAFETY: the slice came from an Init so is initialized
        // and the slice doesn't alias with any pointer given out by
        // the iterator
        unsafe { self.raw.as_slice().as_ptr().drop_in_place() }
    }
}

impl<'a, T> InitIter<'a, T> {
    /// Create a new iterator over Init pointers
    pub fn new(init: Init<'a, [T]>) -> Self {
        Self {
            // SAFETY: the Init is guaranteed to live for at least as long as 'a
            raw: unsafe { RawIter::new(init.into_raw()) },
            _lt: PhantomData,
        }
    }

    /// Get the rest of the slice
    pub fn finish(self) -> Init<'a, [T]> {
        let ptr = self.raw.as_slice();
        core::mem::forget(self);
        // SAFETY: the slice came from an Init so is initialized
        // and the slice doesn't alias with any pointer given out by
        // the iterator
        unsafe { Init::from_raw_nonnull(ptr) }
    }
}

impl<'a, T> ExactSizeIterator for InitIter<'a, T> {}
impl<'a, T> Iterator for InitIter<'a, T> {
    type Item = Init<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let ptr = self.raw.next();
        // SAFETY: this pointer is derived from an Init pointer
        ptr.map(|ptr| unsafe { Init::from_raw_nonnull(ptr) })
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let ptr = self.raw.nth(n);
        // SAFETY: this pointer is derived from an Init pointer
        ptr.map(|ptr| unsafe { Init::from_raw_nonnull(ptr) })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.raw.size_hint()
    }
}
impl<'a, T> DoubleEndedIterator for InitIter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let ptr = self.raw.next_back();
        // SAFETY: this pointer is derived from an Init pointer
        ptr.map(|ptr| unsafe { Init::from_raw_nonnull(ptr) })
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let ptr = self.raw.nth_back(n);
        // SAFETY: this pointer is derived from an Init pointer
        ptr.map(|ptr| unsafe { Init::from_raw_nonnull(ptr) })
    }
}

/// An iterator over uninit pointers
pub struct PinnedUninitIter<'a, T> {
    raw: UninitIter<'a, T>,
}

impl<'a, T> PinnedUninitIter<'a, T> {
    /// Create a new iterator over uninit pointers
    pub fn new(uninit: PinnedUninit<'a, [T]>) -> Self {
        Self {
            // SAFETY: PinnedUninitIter type represents the pinned type-state
            raw: UninitIter::new(unsafe { uninit.into_inner_unchecked() }),
        }
    }

    /// Get the rest of the slice
    pub fn finish(self) -> PinnedUninit<'a, [T]> {
        // SAFETY: the slice came from a `PinnedUninit` so it is in the pinned state
        unsafe { PinnedUninit::new_unchecked(self.raw.finish()) }
    }
}

impl<'a, T> ExactSizeIterator for PinnedUninitIter<'a, T> {}
impl<'a, T> Iterator for PinnedUninitIter<'a, T> {
    type Item = PinnedUninit<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let ptr = self.raw.next();
        // SAFETY: this pointer is derived from an uninit pointer
        ptr.map(|ptr| unsafe { PinnedUninit::new_unchecked(ptr) })
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let ptr = self.raw.nth(n);
        // SAFETY: this pointer is derived from an uninit pointer
        ptr.map(|ptr| unsafe { PinnedUninit::new_unchecked(ptr) })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.raw.size_hint()
    }
}
impl<'a, T> DoubleEndedIterator for PinnedUninitIter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let ptr = self.raw.next_back();
        // SAFETY: this pointer is derived from an uninit pointer
        ptr.map(|ptr| unsafe { PinnedUninit::new_unchecked(ptr) })
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let ptr = self.raw.nth_back(n);
        // SAFETY: this pointer is derived from an uninit pointer
        ptr.map(|ptr| unsafe { PinnedUninit::new_unchecked(ptr) })
    }
}

/// An iterator over init pointers
pub struct PinnedInitIter<'a, T> {
    raw: InitIter<'a, T>,
}

impl<'a, T> PinnedInitIter<'a, T> {
    /// Create a new iterator over init pointers
    pub fn new(init: Pin<Init<'a, [T]>>) -> Self {
        Self {
            // SAFETY: PinnedInitIter type represents the pinned type-state
            raw: InitIter::new(unsafe { Pin::into_inner_unchecked(init) }),
        }
    }

    /// Get the rest of the slice
    pub fn finish(self) -> Pin<Init<'a, [T]>> {
        // SAFETY: the slice came from a `Pin` so it is in the pinned state
        unsafe { Pin::new_unchecked(self.raw.finish()) }
    }
}

impl<'a, T> ExactSizeIterator for PinnedInitIter<'a, T> {}
impl<'a, T> Iterator for PinnedInitIter<'a, T> {
    type Item = Pin<Init<'a, T>>;

    fn next(&mut self) -> Option<Self::Item> {
        let ptr = self.raw.next();
        // SAFETY: this pointer is derived from an pinned init pointer
        ptr.map(|ptr| unsafe { Pin::new_unchecked(ptr) })
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let ptr = self.raw.nth(n);
        // SAFETY: this pointer is derived from an pinned init pointer
        ptr.map(|ptr| unsafe { Pin::new_unchecked(ptr) })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.raw.size_hint()
    }
}
impl<'a, T> DoubleEndedIterator for PinnedInitIter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let ptr = self.raw.next_back();
        // SAFETY: this pointer is derived from an pinned init pointer
        ptr.map(|ptr| unsafe { Pin::new_unchecked(ptr) })
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let ptr = self.raw.nth_back(n);
        // SAFETY: this pointer is derived from an pinned init pointer
        ptr.map(|ptr| unsafe { Pin::new_unchecked(ptr) })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_simple() {
        let mut array = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let ptr = NonNull::from(&mut array);

        // SAFETY: the array is allocated
        let iter = unsafe { RawIter::new(ptr) };

        assert_eq!(iter.len(), array.len());

        for (i, j) in iter.enumerate() {
            // SAFETY: the array is in bounds
            unsafe { assert_eq!(i, *j.as_ptr()) }
        }
    }

    #[test]
    fn test_nth() {
        let mut array = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let ptr = NonNull::from(&mut array);

        // SAFETY: the array is allocated
        let iter = unsafe { RawIter::new(ptr) };

        assert_eq!(iter.len(), array.len());

        for (i, j) in iter.step_by(3).enumerate() {
            // SAFETY: the array is in bounds
            unsafe { assert_eq!(3 * i, *j.as_ptr()) }
        }
    }

    #[test]
    fn test_zero_sized() {
        let mut array = [(); 17];
        let ptr = NonNull::from(&mut array);

        // SAFETY: the array is allocated
        let iter = unsafe { RawIter::new(ptr) };

        assert_eq!(iter.len(), array.len());

        let count = iter.fold(0, |acc, _| acc + 1);
        assert_eq!(count, array.len());
    }

    #[test]
    fn test_zero_sized_nth() {
        let mut array = [(); 17];
        let ptr = NonNull::from(&mut array);

        // SAFETY: the array is allocated
        let iter = unsafe { RawIter::new(ptr) };

        assert_eq!(iter.len(), array.len());

        let count = iter.step_by(3).fold(0, |acc, _| acc + 1);
        // length / 3 round up
        assert_eq!(count, (array.len() + 2) / 3);
    }

    #[test]
    fn test_rev_simple() {
        let mut array = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let ptr = NonNull::from(&mut array);

        // SAFETY: the array is allocated
        let iter = unsafe { RawIter::new(ptr) };

        assert_eq!(iter.len(), array.len());

        for (i, j) in iter.enumerate().rev() {
            // SAFETY: the array is in bounds
            unsafe { assert_eq!(i, *j.as_ptr()) }
        }
    }

    #[test]
    fn test_rev_nth() {
        let mut array = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let ptr = NonNull::from(&mut array);

        // SAFETY: the array is allocated
        let iter = unsafe { RawIter::new(ptr) };

        assert_eq!(iter.len(), array.len());

        for (i, j) in iter.step_by(3).enumerate().rev() {
            // SAFETY: the array is in bounds
            unsafe { assert_eq!(3 * i, *j.as_ptr()) }
        }
    }

    #[test]
    fn test_rev_zero_sized() {
        let mut array = [(); 17];
        let ptr = NonNull::from(&mut array);

        // SAFETY: the array is allocated
        let iter = unsafe { RawIter::new(ptr) };

        assert_eq!(iter.len(), array.len());

        let count = iter.rev().fold(0, |acc, _| acc + 1);
        assert_eq!(count, array.len());
    }

    #[test]
    fn test_rev_zero_sized_nth() {
        let mut array = [(); 17];
        let ptr = NonNull::from(&mut array);

        // SAFETY: the array is allocated
        let iter = unsafe { RawIter::new(ptr) };

        assert_eq!(iter.len(), array.len());

        let count = iter.rev().step_by(3).fold(0, |acc, _| acc + 1);
        // length / 3 round up
        assert_eq!(count, (array.len() + 2) / 3);
    }
}
