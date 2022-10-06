//! slice iterators for [`Uninit`], [`PinnedUninit`], and [`Init`]

use core::{mem, ptr::NonNull};

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
