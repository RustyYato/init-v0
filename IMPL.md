Welcome to `init`, a safe in-place value fallible initialization library! How do we do it?!?!

## Humble beginnings (unsafe soup)

At the heart of in-place initialization we have to pass a pointer to _where_ we want to initialize the value, and the callee
must ensure that the pointer is initialized. For example, with this interface:

```rust
struct SomeSelfReferentialType { ... }

unsafe fn init_me(ptr: *mut SomeSelfReferentialType) {
    ...
}
```

However this suffers a few problems:

- WHY IS IT ALWAYS UNSAFE?

There isn't any way to marshal this into a safe as is. But let's try to make this incrementally more safe,
and we'll see how `init`'s design just falls out form this.

## Maybe Safe?

First, let's remove the raw pointer, and use [`MaybeUninit`].

```rust
struct SomeSelfReferentialType { ... }

fn init_me(ptr: &mut MaybeUninit<SomeSelfReferentialType>) {
    ...
}
```

Wait, we're safe now right? WRONG, how about call-sites?

```rust
let mut slot = MaybeUninit::uninit();
init_me(&mut slot);
let slot = unsafe { slot.assume_init() };
```

Is this safe? We would need to check the docs on `init_me`. What if `init_me` has an error?

We also lost something in the process, how do you handle unsized types? The `T` in `MaybeUninit<T>` must be `Sized`.
I'd say this is a small price to pay for safety, let's punt it for later.

What if we returned an initialized token that guarantees that the slot was initialized?

```rust
struct SomeSelfReferentialType { ... }

fn init_me(ptr: &mut MaybeUninit<SomeSelfReferentialType>) -> &mut SomeSelfReferentialType {
    ...
}
```

So now, if `init_me` returns a `&mut _`, then we know the slot was initialized. We just need to witness the return type.
Great, let's encode this into a trait, so we can abstract over this useful concept. For now, let's ignore error handling, we can always come back to it.

## ~~Maybe Safe?~~ Really this time?

```rust
trait Initialize {
    fn init(slot: &mut MaybeUninit<Self>) -> &mut Self;
}
```

Ok, great, but this isn't ideal. What if we wanted to parameterize initialization. How about custom initializers for downstream users of our types?

```rust
trait Initialize<T> {
    fn init(self, slot: &mut MaybeUninit<T>) -> &mut T;
}
```

There, much better. Now we can pass in arbitrary arguments to `init` and downstream users can write custom initializers for our types.
Just for fun, let's assume we can write a in-place initializer for heap allocations. This should be sound because we can guarantee that
the type was initialized because we witness the `&mut T` returned from init, and can manage the allocation ourselves.

```rust
fn box_in_place<I: Initialize<T>, T>(init: I) -> Box<T> { ... }
```

What could possible go wrong?

```rust
struct NotSus<I>(I);

impl<I: Initialize<T>, T> Initialize<T> for NotSus<I> {
    fn init(self, slot: &mut MaybeUninit<T>) -> &mut T {
        // 😱😱😱😱😱
        Box::leak(box_in_place(self.0))
    }
}
```

What's THIS, we were able to return a reference that doesn't point to the slot?
Oh no.
OH NO.

Now, you may be thinking we should just force callers of `init` to ensure that the pointers are the same, but this doesn't scale.
And also we can do better.

Note: the box is unnecessary to demonstrate unsoundness, you can also use globals and a sort of take once primitive to return a reference to some global instead of a boxed value.

This API is fundamentally unsound.

## Invariant Lifetimes

The problem here is variance, specifically that `&'a mut T` is covariant in `'a`, and that `Box::leak` returns an arbitrary lifetime.
Either one of these will cause issues.

How do we work around covariant lifetimes: with invariant lifetimes
How do we work around `Box::leak` returning an arbitrary lifetime: with an custom pointer type

The invariant lifetimes allow us to establish a owns relationship.

For example, check this out:

```rust
struct Invariant<'a>(&'a mut &'a mut ());

pub struct Start<'a>(PhantomData<Invariant<'a>>);
pub struct End<'a>(PhantomData<Invariant<'a>>);

impl<'a> Start<'a> {
    pub fn finish(self) -> End<'a> {
        End(self.0)
    }
}

pub fn handle<F: FnOnce(Start<'_>) -> End<'_>>(f: F) {
    f(Start(PhantomData))
}
```

All closure passed to `handle` will be guaranteed to call `Start::finish` or panic, and the `End` came from the start that was passed into the closure because

- the lifetime is invariant so you can't abuse subtypes.
- there are no other ways to convert from `Start` to `End`

This is exactly what we want, because this allows us to ensure that the pointer that we pass to `init` is returned from `init`.

## Truly Safe

So here's the next step:

```rust
pub struct Uninit<'a, T> {
    ptr: &'a mut MaybeUninit<T>,
    _inv: PhantomData<Invariant<'a>>,
}

pub struct Init<'a, T> {
    ptr: &'a mut T,
    _inv: PhantomData<Invariant<'a>>,
}

trait Initialize<T> {
    fn init(self, slot: Uninit<'_, T>) -> Init<'_, T>;
}
```

Now we have a concrete guarantee that the slot is returned from `Initialize::init` _at compile time_. This is the same
feature that underpins [`generativity`](https://crates.io/crates/generativity).

Now this is great and all, but what about fallible initialization? I was promised fallible initialization.

```rust
trait TryInitialize<T> {
    type Error;

    fn try_init(self, slot: Uninit<'_, T>) -> Result<Init<'_, T>, Self::Error>;
}
```

What about unsized types?

```rust
pub struct Uninit<'a, T> {
    ptr: NonNull<T>,
    _inv: PhantomData<(Invariant<'a>)>,
}

pub struct Init<'a, T: ?Sized> {
    ptr: NonNull<T>,
    _inv: PhantomData<Invariant<'a>>,
}
```

And this is basically the design for the core pointer types and the interface for `init`.
There are a few more details, and you can check out the source code for those details.

What about allowing both errors and initialization at the same time? That's out of scope for this library.
