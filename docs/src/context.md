# Contexts

Contexts contains the state of your Stores. You rarely (if ever) need to manage them manually, but
it's useful to understand how they work.

You can easily create a new local context with `Context::new`. Then just pass it into a dispatch and
you have your very own locally managed store!

```rust
# extern crate yew;
# extern crate yewdux;
use yew::prelude::*;
use yewdux::prelude::*;

#[derive(Clone, PartialEq, Default, Store)]
struct Counter(u32);

let cx = yewdux::Context::new();
let dispatch = Dispatch::<Counter>::new(&cx);
```

Changes to one context are not reflected in any others:

```rust
# extern crate yewdux;
# use yewdux::prelude::*;
# #[derive(Clone, PartialEq, Default, Store)]
# struct Counter(u32);
let cx_1 = yewdux::Context::new();
let dispatch_1 = Dispatch::<Counter>::new(&cx_1);

let cx_2 = yewdux::Context::new();
let dispatch_2 = Dispatch::<Counter>::new(&cx_2);

dispatch_1.set(Counter(1));
dispatch_2.set(Counter(2));

assert!(dispatch_1.get() != dispatch_2.get());
```

## The Global Context

You may already be familar with the global context. This is what you are using when you create a
dispatch with `Dispatch::global`. The global context is thread-local, meaning you can access it from
anywhere in your code as long as it's on the same thread (for wasm this is effectively everywhere).

```rust
# extern crate yewdux;
# use yewdux::prelude::*;
# #[derive(Clone, PartialEq, Default, Store)]
# struct Counter(u32);
// These are equivalent!
let dispatch_1 = Dispatch::<Counter>::global();
let dispatch_2 = Dispatch::<Counter>::new(&yewdux::Context::global());

dispatch_1.set(Counter(1));

assert!(dispatch_1.get() == dispatch_2.get());
```

**IMPORTANT**: Use of global context is only supported for wasm targets. See [ssr support](./ssr.md)
for more details.
-------

