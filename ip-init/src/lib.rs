//! Init is a crate that handles fallible in-place initialization

#![feature(slice_ptr_len, dropck_eyepatch, strict_provenance)]
#![forbid(
    clippy::undocumented_unsafe_blocks,
    clippy::missing_safety_doc,
    unsafe_op_in_unsafe_fn,
    missing_docs,
    clippy::missing_panics_doc
)]
#![no_std]

#[cfg(feature = "alloc")]
#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(feature = "std")]
extern crate std;
#[cfg(feature = "std")]
extern crate std as alloc;

#[doc(hidden)]
pub mod macros;

mod ptr;

pub use pin_ptr::{PinnedInit, PinnedUninit};
pub use ptr::{Init, Uninit};
pub mod pin_ptr;

pub mod func;
pub mod raw;
pub mod traits;

pub mod array;
pub mod iter;
pub mod slice;

pub mod layout;

#[cfg(feature = "alloc")]
pub mod boxed;

pub mod pin;
