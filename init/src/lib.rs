//! Init is a crate that handles fallible in-place initialization

#![feature(slice_ptr_len, dropck_eyepatch)]
#![forbid(
    clippy::undocumented_unsafe_blocks,
    clippy::missing_safety_doc,
    unsafe_op_in_unsafe_fn,
    missing_docs,
    clippy::missing_panics_doc
)]
#![no_std]

mod ptr;

use pin::PinnedUninit;
pub use ptr::{Init, Uninit};
pub mod pin;

pub mod func;
pub mod raw;
pub mod traits;

pub mod iter;
pub mod slice;
