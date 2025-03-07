# Derived State

Derived state allows you to create state that automatically reacts to changes in another store. This is useful for:

- Computing derived values from your primary state
- Creating focused views of larger state objects
- Building dependent state relationships

## Defining Derived State

There are two ways to create derived state:

1. Using the `Store` macro with `derived_from` or `derived_from_mut` attributes
2. Manually implementing the `Store` trait and calling `derived_from` or `derived_from_mut`

### Using the Store Macro

The simplest approach is to use the `Store` derive macro with the `derived_from` or `derived_from_mut` attributes:

```rust
use std::rc::Rc;
use yewdux::prelude::*;

// Original source state
#[derive(Default, Clone, PartialEq, Store)]
struct Count {
    count: u32,
}

// Immutable derived state - creates a new instance on change
#[derive(Default, Clone, PartialEq, Store)]
#[store(derived_from(Count))]
struct CountMultiplied {
    value: u32,
}

impl DerivedFrom<Count> for CountMultiplied {
    fn on_change(&self, state: Rc<Count>) -> Self {
        Self {
            value: state.count * 10,
        }
    }
}

// Mutable derived state - updates in place
#[derive(Default, Clone, PartialEq, Store)]
#[store(derived_from_mut(Count))]
struct CountIsEven {
    status: bool,
}

impl DerivedFromMut<Count> for CountIsEven {
    fn on_change(&mut self, state: Rc<Count>) {
        self.status = state.count % 2 == 0;
    }
}
```

### Manual Implementation

For more control, you can implement `Store` manually and register the relationship in your `new` method:

```rust
#[derive(Default, Clone, PartialEq)]
struct CountIsEven {
    status: bool,
}

impl DerivedFromMut<Count> for CountIsEven {
    fn on_change(&mut self, state: Rc<Count>) {
        self.status = state.count % 2 == 0;
    }
}

impl Store for CountIsEven {
    fn new(cx: &yewdux::Context) -> Self {
        // Register this state as derived from `Count`
        cx.derived_from_mut::<Count, Self>();

        // Initialize with current Count value
        let status = cx.get::<Count>().count % 2 == 0;
        Self { status }
    }

    fn should_notify(&self, old: &Self) -> bool {
        self != old
    }
}
```

## Using Derived State

Using derived state is identical to using any other store:

```rust
#[function_component]
fn App() -> Html {
    let (count, dispatch) = use_store::<Count>();
    let is_even = use_store_value::<CountIsEven>();
    let multiplied = use_store_value::<CountMultiplied>();
    
    let onclick = dispatch.reduce_mut_callback(|state| state.count += 1);

    html! {
        <>
        <p>{"Count: "}{ count.count }</p>
        <p>{"Is Even: "}{ is_even.status.to_string() }</p>
        <p>{"Multiplied by 10: "}{ multiplied.value }</p>
        <button {onclick}>{"+1"}</button>
        </>
    }
}
```

## How It Works

When you use `derived_from` or `derived_from_mut`:

1. A listener is registered that watches for changes in the source state
2. When the source state changes, your `on_change` implementation is called
3. Your derived state is updated either by creating a new instance (`DerivedFrom`) or by modifying it in place (`DerivedFromMut`)
4. Components using the derived state are re-rendered

This provides a clean, type-safe way to create computed or dependent state without manual synchronization.