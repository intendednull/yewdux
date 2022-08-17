# Selectors

Sometimes a component will only care about a particular part of state, and only needs to re-render
when that part changes. For this we have the `use_selector` hook.

```rust
#[derive(Default, Clone, PartialEq, Store)]
struct User {
    first_name: String,
    last_name: String,
}

#[function_component]
fn DisplayFirst() -> Html {
    // This will only re-render when the first name has changed. It will **not** re-render if any
    // other field has changed.
    //
    // Note: we are cloning a string. Probably insignificant for this example, however
    // sometimes it may be beneficial to wrap fields that are expensive to clone in an `Rc`.
    let first_name = use_selector(|state: &User| state.first_name.clone());

    html! {
        <p>{ first_name }</p>
    }
}
```

## Capturing your environment

For selectors that need to capture variables from their environment, be sure to provide them as
dependencies to `use_selector_with_deps`. Otherwise your selector won't update correctly!

```rust
#[derive(Default, Clone, PartialEq, Store)]
struct Items {
    inner: HashMap<u32, String>,
}

#[derive(Clone, PartialEq, Properties)]
struct DisplayItemProps {
    item_id: u32,
}

#[function_component]
fn DisplayItem(props: &DisplayItemProps) -> Html {
    // For multiple dependencies, try using a tuple: (dep1, dep2, ..)
    let item = use_selector_with_deps(
        |state: &Items, item_id| state.inner.get(item_id).cloned(),
        props.item_id,
    );
    // Only render the item if it exists.
    let item = match item {
        Some(item) => item,
        None => return Default::default(),
    };

    html! {
        <p>{ item }</p>
    }
}
```
