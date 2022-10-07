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
