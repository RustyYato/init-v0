use std::{marker::PhantomPinned, pin::Pin};

use init::{pin_uninit::PinnedUninit, traits::PinInitialize, Init};

#[pin_project::pin_project]
pub struct SelfRef {
    first: i32,
    second: i32,

    current: *mut i32,
    _pin: PhantomPinned,
}

unsafe impl Send for SelfRef {}
unsafe impl Sync for SelfRef {}

impl SelfRef {
    pub fn init(value: i32) -> impl PinInitialize<Self> {
        init::func::PinInitFn::new(move |uninit| Self::new_in(value, uninit))
    }

    pub fn new_in(value: i32, mut uninit: PinnedUninit<Self>) -> Pin<Init<Self>> {
        let ptr = uninit.as_mut_ptr();

        let current = unsafe { core::ptr::addr_of_mut!((*ptr).first) };

        uninit.write(Self {
            first: value,
            second: 0,
            current,
            _pin: PhantomPinned,
        })
    }

    pub fn new(value: i32) -> Pin<Box<Self>> {
        init::boxed::emplace_pin(init::layout::SizedLayoutProvider, Self::init(value))
    }

    pub fn get(self: Pin<&Self>) -> i32 {
        unsafe { *self.current }
    }

    pub fn set_first(self: Pin<&mut Self>) {
        let this = self.project();
        *this.current = &mut *this.first;
    }

    pub fn set_second(self: Pin<&mut Self>) {
        let this = self.project();
        *this.current = &mut *this.second;
    }
}

#[test]
fn test() {
    let mut this = SelfRef::new(10);

    let a = this.as_ref().get();
    this.as_mut().set_second();
    let b = this.as_ref().get();
    this.as_mut().set_first();
    let c = this.as_ref().get();
    assert_eq!(a, 10);
    assert_eq!(b, 0);
    assert_eq!(c, 10);
}
