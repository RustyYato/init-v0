pub use core;

// SAFETY: INTERNAL to project
pub unsafe fn bind<'a, T: ?Sized, U: ?Sized>(
    _: &'a mut crate::Uninit<T>,
    ptr: *mut U,
) -> crate::Uninit<'a, U> {
    // SAFETY: only used in project
    unsafe { crate::Uninit::from_raw(ptr) }
}

// SAFETY: INTERNAL to project
pub unsafe fn bind_pin<'a, T: ?Sized, U: ?Sized>(
    _: &'a mut crate::PinnedUninit<T>,
    ptr: *mut U,
) -> crate::PinnedUninit<'a, U> {
    // SAFETY: only used in project
    unsafe { crate::PinnedUninit::new_unchecked(crate::Uninit::from_raw(ptr)) }
}

/// Create an uninit stack slot
#[macro_export]
macro_rules! slot {
    ($name:ident : $($type:ty)?) => {
        let mut $name = $crate::macros::core::mem::MaybeUninit$(::<$type>)?::uninit();
        let $name = $crate::Uninit::from_maybe_uninit(&mut $name);
    };
}

/// Create an pinned uninit stack slot
#[macro_export]
macro_rules! slot_pin {
    ($name:ident : $($type:ty)?) => {
        let mut $name = $crate::macros::core::mem::MaybeUninit$(::<$type>)?::uninit();
        let $name = unsafe { $crate::macros::core::pin::Pin::new_unchecked(&mut $name) };
        let $name = $crate::PinnedUninit::from_maybe_uninit($name);
    };
}

/// Project a uninit ptr to one of it's fields
#[macro_export]
macro_rules! project {
    ($type:path, $uninit:expr, $field:ident) => {
        match $uninit {
            ref mut uninit => {
                let _: $crate::Uninit<$type> = *uninit;

                if false {
                    unsafe {
                        let $type { $field: _, .. } = (*uninit.as_mut_ptr());
                    }
                }

                let projected = unsafe {
                    $crate::macros::core::ptr::addr_of_mut!((*uninit.as_mut_ptr()).$field)
                };

                unsafe { $crate::macros::bind(uninit, projected) }
            }
        }
    };
}

/// Project a uninit ptr to one of it's fields
#[macro_export]
macro_rules! project_pin {
    ($type:path, $uninit:expr, $field:ident) => {
        match $uninit {
            ref mut uninit => {
                let _: $crate::PinnedUninit<$type> = *uninit;

                if false {
                    unsafe {
                        let $type { $field: _, .. } = *uninit.as_mut_ptr();
                    }
                }

                let projected = unsafe {
                    $crate::macros::core::ptr::addr_of_mut!((*uninit.as_mut_ptr()).$field)
                };

                unsafe { $crate::macros::bind_pin(uninit, projected) }
            }
        }
    };
}
