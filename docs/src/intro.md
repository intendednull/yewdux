# Yewdux

Simple state management for [Yew](https://yew.rs) applications.

This crate tries to provide a dead-simple, zero-cost approach to global state management. It does
*not* try to provide any additional patterns or features which aren't directly related to accessing
or manipulating shared state.

Some key features include:
- Zero-clone - user has complete control over how state is changed. Yewdux will never deep-copy your
    state unless explicitly told to. CoW behavior is provided by the `Dispatch::reduce_mut*`
    variants (marked by a `Clone` trait requirement).
- Selective rendering - subscribers are only notified when state has changed, avoiding any
    unnecessary re-renders. Can be further optimized with `use_selector` hooks.
- Access from anywhere - users can create a dispatch to access a store from anywhere, they are not
    restricted to only inside components. This boasts greater flexibility over application flow and
    setup.
- Ergonomic interface - accessing a store is as simple as creating a dispatch with your desired
    store type. From this dispatch you can modify the store directly, create callbacks to trigger
    from events, or even execute in an async context.
- Minimal trait requirements - The only trait required for a type to be a store is the `Store` trait
    itself. While the `Store` macro does need `Default` and `PartialEq` to work, it is also very
    simple to implement `Store` yourself, no additional requirements necessary!
- Complete component support - Yewdux supports both struct components and functional components.
    Although functional is usually the most convenient option, the utility and flexibility of struct
    components cannot be denied.

## Alternatives

- [Bounce](https://github.com/futursolo/bounce) - The uncomplicated Yew State management library
