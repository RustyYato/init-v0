//! Layout providers

use core::alloc::{Layout, LayoutError};

use crate::traits::LayoutProvider;

/// a layout provider for sized types
pub struct SizedLayoutProvider;

// SAFETY: this implementation always returns an error, which is fine
unsafe impl<T> LayoutProvider<T> for SizedLayoutProvider {
    #[inline]
    fn layout_for(&self) -> Result<Layout, LayoutError> {
        // 0 is never a valid alignment, so this always fails
        Ok(Layout::new::<T>())
    }

    #[inline]
    fn cast(&self, ptr: *mut u8) -> *mut T {
        ptr.cast()
    }
}

/// a layout provider for sized types
pub struct SliceLen(pub usize);

// SAFETY: this implementation always returns an error, which is fine
unsafe impl<T> LayoutProvider<[T]> for SliceLen {
    #[inline]
    fn layout_for(&self) -> Result<Layout, LayoutError> {
        Layout::array::<T>(self.0)
    }

    #[inline]
    fn cast(&self, ptr: *mut u8) -> *mut [T] {
        core::ptr::slice_from_raw_parts_mut(ptr.cast(), self.0)
    }
}
