# Setting default store values

The best way to define the default value of your store is by manually implementing `Default`.

```rust
# extern crate yewdux;
# use yewdux::prelude::*;
#[derive(PartialEq, Store)]
struct MyStore {
    foo: String,
    bar: String,
}

impl Default for MyStore {
    fn default() -> Self {
        Self {
            foo: "foo".to_string(),
            bar: "bar".to_string(),
        }
    }
}
```

Sometimes you may need additional context to set the initial value of your store. To do this, there
are a couple options.

You can set the value at the beginning of your application, before your app renders (like in your
main function).

```rust
# extern crate yewdux;
# use yewdux::prelude::*;
# #[derive(PartialEq, Store, Default)]
# struct MyStore {
#     foo: String,
#     bar: String,
# }
fn main() {
    // Construct foo and bar however necessary
    let foo = "foo".to_string();
    let bar = "bar".to_string();
    // Run this before starting your app.
    Dispatch::<MyStore>::global().set(MyStore { foo, bar });
    // ... continue with your app setup
}
```

You can also set the inital value from a function component. The `use_effect_with` hook can be used
to run the hook only once (just be sure to use empty deps).

```rust
# extern crate yew;
# extern crate yewdux;
#
# use yewdux::prelude::*;
# use yew::prelude::*;
# #[derive(PartialEq, Store, Default)]
# struct MyStore {
#     foo: String,
#     bar: String,
# }
#[function_component]
fn MyComponent() -> Html {
    let dispatch = use_dispatch::<MyStore>();
    // This runs only once, on the first render of the component.
    use_effect_with(
        (), // empty deps
        move |_| {
            // Construct foo and bar however necessary
            let foo = "foo".to_string();
            let bar = "bar".to_string();
            dispatch.set(MyStore { foo, bar });
            || {}
        },
    );

    html! {
        // Your component html
    }
}
```

Keep in mind your store will still be initialized with `Store::new` (usually that's set to
`Default::default()`), however this is typically inexpensive.
