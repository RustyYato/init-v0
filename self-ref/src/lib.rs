use std::{marker::PhantomPinned, pin::Pin};

use ip_init::{pin_uninit::PinnedUninit, traits::PinInitialize, Init};

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
        ip_init::func::PinInitFn::new(move |uninit| Self::new_in(value, uninit))
    }

    pub fn new_in(value: i32, mut uninit: PinnedUninit<Self>) -> Pin<Init<Self>> {
        let current = ip_init::project_pin!(Self, uninit, first).as_mut_ptr();

        uninit.write(Self {
            first: value,
            second: 0,
            current,
            _pin: PhantomPinned,
        })
    }

    pub fn new(value: i32) -> Pin<Box<Self>> {
        ip_init::boxed::emplace_pin(ip_init::layout::SizedLayoutProvider, Self::init(value))
    }

    pub fn many(value: i32, count: usize) -> Pin<Box<[Self]>> {
        ip_init::boxed::emplace_pin(
            ip_init::layout::SliceLayoutProvider(count),
            ip_init::func::PinInitFn::new(|uninit| {
                let mut value = value;
                let mut writer = ip_init::slice::PinSliceWriter::new(uninit);

                while !writer.is_finished() {
                    writer.init(Self::init(value));
                    value += 1;
                }

                writer.finish()
            }),
        )
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
