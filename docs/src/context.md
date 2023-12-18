# Contexts

A `Context` contains the state of `Store`s.

You can easily create and use a local context by instantiating with `Context::new`, and creating a
dispatch using your new context.

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

# The Global Context

You may already be familar with the global context. This is what you are using when you create a
dispatch with `Dispatch::global`. The global context is thread-local, and can be accessed easily
from anywhere.

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

