# Mutate state predictably 

Yewdux supports predictable mutation. Simply define your message and apply it.

```rust
struct Msg {
    AddOne,
}

impl Reducer<Counter> for Msg {
    fn apply(&self, counter: Rc<Counter>) -> Rc<Counter> {
        match self {
            Msg::AddOne => Counter { count: counter.count + 1 },
        }
    }
}
```

### Tip 

`Rc::make_mut` is handy if you prefer CoW:

```rust
impl Reducer<Counter> for Msg {
    fn apply(&self, mut counter: Rc<Counter>) -> Rc<Counter> {
        let state = Rc::make_mut(&mut counter);

        match self {
            Msg::AddOne => state.count += 1,
        };

        counter
    }
}
```


## `Dispatch::apply` 

Execute immediately.

```rust
dispatch.apply(Msg::AddOne);
```

## `Dispatch::apply_callback` 

Generate (you guessed it) a callback.

```rust
let onclick = dispatch.apply_callback(|_| Msg::AddOne);
html! {
    <button {onclick}>{"Increment (+1)"}</button>
}
```

