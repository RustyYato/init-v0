//! create and initialize heap allocations in place

use core::{
    alloc::{Layout, LayoutError},
    fmt::Debug,
    pin::Pin,
    ptr::NonNull,
};

use ::alloc::{alloc, boxed::Box};

use crate::traits::{LayoutProvider, TryInitialize, TryPinInitialize};

/// Emplace failure error
pub enum AllocError<E> {
    /// Initialization failed
    Init(E),
    /// The layout could not be computed
    Layout(LayoutError),
    /// The allocation failed with the given layout
    Alloc(Layout),
}

#[cold]
#[inline(never)]
fn handle<E: Debug>(err: &AllocError<E>) -> ! {
    match err {
        AllocError::Init(err) => panic!("Failed to initialize value: {err:?}"),
        AllocError::Layout(_) => panic!("Could not compute layout for value"),
        AllocError::Alloc(layout) => alloc::handle_alloc_error(*layout),
    }
}

/// create a new T, and initialize it in place
pub fn emplace<T: ?Sized, L, I>(provider: L, init: I) -> Box<T>
where
    I: TryInitialize<T>,
    L: LayoutProvider<T>,
    I::Error: Debug,
{
    match try_emplace(provider, init) {
        Ok(boxed) => boxed,
        Err(ref err) => handle(err),
    }
}

/// create a new T, and initialize it in place
pub fn emplace_pin<T: ?Sized, L, I>(provider: L, init: I) -> Pin<Box<T>>
where
    I: TryPinInitialize<T>,
    L: LayoutProvider<T>,
    I::Error: Debug,
{
    match try_emplace_pin(provider, init) {
        Ok(boxed) => boxed,
        Err(ref err) => handle(err),
    }
}

/// create a new T, and initialize it in place
pub fn try_emplace<T: ?Sized, L, I>(provider: L, init: I) -> Result<Box<T>, AllocError<I::Error>>
where
    I: TryInitialize<T>,
    L: LayoutProvider<T>,
{
    struct RawAllocation {
        ptr: *mut u8,
        layout: Layout,
    }

    impl Drop for RawAllocation {
        fn drop(&mut self) {
            // SAFETY: RawAllocation is only constructed with a ptr allocated from
            // the global allocator with the given layout. So it's safe to deallocate it
            // using the same layout
            unsafe { alloc::dealloc(self.ptr, self.layout) }
        }
    }

    let layout = match provider.layout_for() {
        Ok(layout) => layout,
        Err(err) => return Err(AllocError::Layout(err)),
    };

    let ptr = if layout.size() == 0 {
        layout.align() as *mut u8
    } else {
        // SAFETY: the layout has non-zero size
        unsafe { alloc::alloc(layout) }
    };

    let ptr = match NonNull::new(ptr) {
        Some(ptr) => provider.cast_nonnull(ptr),
        None => return Err(AllocError::Alloc(layout)),
    };

    let alloc = RawAllocation {
        ptr: ptr.cast().as_ptr(),
        layout,
    };

    // SAFETY: the pointer is allocated for T (`LayoutProvider`), and valid for
    // is valid for writes and reads (after writes)
    match unsafe { crate::raw::try_init_in_place(init, ptr.as_ptr()) } {
        Ok(()) => (),
        Err(err) => return Err(AllocError::Init(err)),
    }

    core::mem::forget(alloc);

    // SAFETY: the pointer is now initialized and allocated via the global allocator
    Ok(unsafe { Box::from_raw(ptr.as_ptr()) })
}

/// create a new T, and initialize it in place
pub fn try_emplace_pin<T: ?Sized, L, I>(
    provider: L,
    init: I,
) -> Result<Pin<Box<T>>, AllocError<I::Error>>
where
    I: TryPinInitialize<T>,
    L: LayoutProvider<T>,
{
    struct RawAllocation {
        ptr: *mut u8,
        layout: Layout,
    }

    impl Drop for RawAllocation {
        fn drop(&mut self) {
            // SAFETY: RawAllocation is only constructed with a ptr allocated from
            // the global allocator with the given layout. So it's safe to deallocate it
            // using the same layout
            unsafe { alloc::dealloc(self.ptr, self.layout) }
        }
    }

    let layout = match provider.layout_for() {
        Ok(layout) => layout,
        Err(err) => return Err(AllocError::Layout(err)),
    };

    let ptr = if layout.size() == 0 {
        layout.align() as *mut u8
    } else {
        // SAFETY: the layout has non-zero size
        unsafe { alloc::alloc(layout) }
    };

    let ptr = match NonNull::new(ptr) {
        Some(ptr) => provider.cast_nonnull(ptr),
        None => return Err(AllocError::Alloc(layout)),
    };

    let alloc = RawAllocation {
        ptr: ptr.cast().as_ptr(),
        layout,
    };

    // SAFETY: the pointer is allocated for T (`LayoutProvider`), and valid for
    // is valid for writes and reads (after writes)
    // the value is kept in the pinned type-state
    match unsafe { crate::raw::try_pin_init_in_place(init, ptr.as_ptr()) } {
        Ok(()) => (),
        Err(err) => return Err(AllocError::Init(err)),
    }

    core::mem::forget(alloc);

    // SAFETY: the pointer is now initialized and allocated via the global allocator
    let boxed = unsafe { Box::from_raw(ptr.as_ptr()) };

    // It's always safe to pin a Box<T>
    Ok(Box::into_pin(boxed))
}
