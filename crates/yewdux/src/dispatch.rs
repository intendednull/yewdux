//!  This module defines how you can interact with your [`Store`].
//!
//!  ```ignore
//!  use yewdux::prelude::*;
//!
//!  #[derive(Default, Clone, Store)]
//!  struct MyState {
//!     count: usize,
//!  }
//!
//!  # fn main() {
//!  let dispatch = Dispatch::<MyState>::new();
//!  dispatch.reduce(|state| state.count = 1);
//!
//!  let state = dispatch.get();
//!
//!  assert!(state.count == 1);
//!  # }
//!  ```
//!

use std::rc::Rc;
#[cfg(feature = "future")]
use std::{future::Future, pin::Pin};

use yew::Callback;

use anyflux::{
    dispatch::Dispatch,
    store::{Reducer, Store},
};

pub trait DispatchExt<S> {
    /// Callback for sending a message to the store.
    ///
    /// ```ignore
    /// let onclick = dispatch.apply_callback(|_| StoreMsg::AddOne);
    /// ```
    fn apply_callback<E, M, F>(&self, f: F) -> Callback<E>
    where
        M: Reducer<S>,
        F: Fn(E) -> M + 'static;

    /// Set state using value from callback.
    fn set_callback<E, F>(&self, f: F) -> Callback<E>
    where
        F: Fn(E) -> S + 'static;

    /// Like [reduce](Self::reduce) but from a callback.
    ///
    /// ```ignore
    /// let onclick = dispatch.reduce_callback(|s| State { count: s.count + 1 });
    /// ```
    fn reduce_callback<F, R, E>(&self, f: F) -> Callback<E>
    where
        R: Into<Rc<S>>,
        F: Fn(Rc<S>) -> R + 'static,
        E: 'static;

    /// Create a callback to reduce state asynchronously.
    ///
    ///  ```ignore
    /// let incr = dispatch.reduce_future_callback(|state| async move {
    ///     State {
    ///         count: state.count + 1,
    ///     }
    /// });
    /// ```
    ///
    #[cfg(feature = "future")]
    fn reduce_future_callback<R, FUT, FUN, E>(&self, f: FUN) -> Callback<E>
    where
        R: Into<Rc<S>>,
        FUT: Future<Output = R>,
        FUN: Fn(Rc<S>) -> FUT + 'static,
        E: 'static;

    /// Similar to [Self::reduce_callback] but also provides the fired event.
    ///
    /// ```ignore
    /// let oninput = dispatch.reduce_callback_with(|state, count: u32| State { count });
    /// ```
    fn reduce_callback_with<F, R, E>(&self, f: F) -> Callback<E>
    where
        R: Into<Rc<S>>,
        F: Fn(Rc<S>, E) -> R + 'static,
        E: 'static;

    /// Create a callback to reduce state asynchronously, with the fired event.
    ///
    /// ```ignore
    /// let incr = dispatch.reduce_future_callback_with(|state, count| async move {
    ///     State {
    ///         count: state.count + count,
    ///     }
    /// });
    /// ```
    ///
    #[cfg(feature = "future")]
    fn reduce_future_callback_with<R, FUT, FUN, E>(&self, f: FUN) -> Callback<E>
    where
        R: Into<Rc<S>>,
        FUT: Future<Output = R>,
        FUN: Fn(Rc<S>, E) -> FUT + 'static,
        E: 'static;

    /// Like [Self::reduce_mut] but from a callback.
    ///
    /// ```ignore
    /// let onclick = dispatch.reduce_mut_callback(|s| s.count += 1);
    /// ```
    fn reduce_mut_callback<F, R, E>(&self, f: F) -> Callback<E>
    where
        S: Clone,
        F: Fn(&mut S) -> R + 'static,
        E: 'static;

    /// Create a callback to asynchronously mutate state with given function.
    ///
    /// ```ignore
    /// let incr = dispatch.reduce_mut_future_callback(|state| Box::pin(async move {
    ///     state.count += 1;
    /// }));
    /// ```
    ///
    #[cfg(feature = "future")]
    fn reduce_mut_future_callback<R, F, E>(&self, f: F) -> Callback<E>
    where
        S: Clone,
        F: Fn(&mut S) -> Pin<Box<dyn Future<Output = R> + '_>> + 'static,
        E: 'static;

    /// Similar to [Self::reduce_mut_callback] but also provides the fired event.
    ///
    /// ```ignore
    /// let oninput = dispatch.reduce_mut_callback_with(|state, name: String| state.name = name);
    /// ```
    fn reduce_mut_callback_with<F, R, E>(&self, f: F) -> Callback<E>
    where
        S: Clone,
        F: Fn(&mut S, E) -> R + 'static,
        E: 'static;

    /// Create a callback to asynchronously mutate state with given function, provided the fired
    /// event.
    ///
    /// ```ignore
    /// let incr = dispatch.reduce_mut_future_callback_with(|state, count| Box::pin(async move {
    ///     state.count += count;
    /// }));
    /// ```
    ///
    #[cfg(feature = "future")]
    fn reduce_mut_future_callback_with<R, F, E>(&self, f: F) -> Callback<E>
    where
        S: Clone,
        F: Fn(&mut S, E) -> Pin<Box<dyn Future<Output = R> + '_>> + 'static,
        E: 'static;
}

impl<S: Store> DispatchExt<S> for Dispatch<S> {
    /// Callback for sending a message to the store.
    ///
    /// ```ignore
    /// let onclick = dispatch.apply_callback(|_| StoreMsg::AddOne);
    /// ```
    fn apply_callback<E, M, F>(&self, f: F) -> Callback<E>
    where
        M: Reducer<S>,
        F: Fn(E) -> M + 'static,
    {
        Callback::from(move |e| {
            let msg = f(e);
            Self::new().apply(msg);
        })
    }

    /// Set state using value from callback.
    fn set_callback<E, F>(&self, f: F) -> Callback<E>
    where
        F: Fn(E) -> S + 'static,
    {
        Callback::from(move |e| {
            let val = f(e);
            Self::new().set(val);
        })
    }

    /// Like [reduce](Self::reduce) but from a callback.
    ///
    /// ```ignore
    /// let onclick = dispatch.reduce_callback(|s| State { count: s.count + 1 });
    /// ```
    fn reduce_callback<F, R, E>(&self, f: F) -> Callback<E>
    where
        R: Into<Rc<S>>,
        F: Fn(Rc<S>) -> R + 'static,
        E: 'static,
    {
        Callback::from(move |_| {
            Self::new().reduce(&f);
        })
    }

    /// Create a callback to reduce state asynchronously.
    ///
    ///  ```ignore
    /// let incr = dispatch.reduce_future_callback(|state| async move {
    ///     State {
    ///         count: state.count + 1,
    ///     }
    /// });
    /// ```
    ///
    #[cfg(feature = "future")]
    fn reduce_future_callback<R, FUT, FUN, E>(&self, f: FUN) -> Callback<E>
    where
        R: Into<Rc<S>>,
        FUT: Future<Output = R>,
        FUN: Fn(Rc<S>) -> FUT + 'static,
        E: 'static,
    {
        let f = Rc::new(f);
        Callback::from(move |_| {
            let f = f.clone();
            wasm_bindgen_futures::spawn_local(async move {
                Self::new().reduce_future(f.as_ref()).await;
            })
        })
    }

    /// Similar to [Self::reduce_callback] but also provides the fired event.
    ///
    /// ```ignore
    /// let oninput = dispatch.reduce_callback_with(|state, count: u32| State { count });
    /// ```
    fn reduce_callback_with<F, R, E>(&self, f: F) -> Callback<E>
    where
        R: Into<Rc<S>>,
        F: Fn(Rc<S>, E) -> R + 'static,
        E: 'static,
    {
        Callback::from(move |e: E| {
            Self::new().reduce(|x| f(x, e));
        })
    }

    /// Create a callback to reduce state asynchronously, with the fired event.
    ///
    /// ```ignore
    /// let incr = dispatch.reduce_future_callback_with(|state, count| async move {
    ///     State {
    ///         count: state.count + count,
    ///     }
    /// });
    /// ```
    ///
    #[cfg(feature = "future")]
    fn reduce_future_callback_with<R, FUT, FUN, E>(&self, f: FUN) -> Callback<E>
    where
        R: Into<Rc<S>>,
        FUT: Future<Output = R>,
        FUN: Fn(Rc<S>, E) -> FUT + 'static,
        E: 'static,
    {
        let f = Rc::new(f);
        Callback::from(move |e: E| {
            let f = f.clone();
            wasm_bindgen_futures::spawn_local(async move {
                Self::new().reduce_future(move |s| f(s, e)).await;
            })
        })
    }

    /// Like [Self::reduce_mut] but from a callback.
    ///
    /// ```ignore
    /// let onclick = dispatch.reduce_mut_callback(|s| s.count += 1);
    /// ```
    fn reduce_mut_callback<F, R, E>(&self, f: F) -> Callback<E>
    where
        S: Clone,
        F: Fn(&mut S) -> R + 'static,
        E: 'static,
    {
        Callback::from(move |_| {
            Self::new().reduce_mut(|x| {
                f(x);
            });
        })
    }

    /// Create a callback to asynchronously mutate state with given function.
    ///
    /// ```ignore
    /// let incr = dispatch.reduce_mut_future_callback(|state| Box::pin(async move {
    ///     state.count += 1;
    /// }));
    /// ```
    ///
    #[cfg(feature = "future")]
    fn reduce_mut_future_callback<R, F, E>(&self, f: F) -> Callback<E>
    where
        S: Clone,
        F: Fn(&mut S) -> Pin<Box<dyn Future<Output = R> + '_>> + 'static,
        E: 'static,
    {
        let f = Rc::new(f);
        Callback::from(move |_| {
            let f = f.clone();
            wasm_bindgen_futures::spawn_local(async move {
                Self::new().reduce_mut_future(f.as_ref()).await;
            })
        })
    }

    /// Similar to [Self::reduce_mut_callback] but also provides the fired event.
    ///
    /// ```ignore
    /// let oninput = dispatch.reduce_mut_callback_with(|state, name: String| state.name = name);
    /// ```
    fn reduce_mut_callback_with<F, R, E>(&self, f: F) -> Callback<E>
    where
        S: Clone,
        F: Fn(&mut S, E) -> R + 'static,
        E: 'static,
    {
        Callback::from(move |e: E| {
            Self::new().reduce_mut(|x| {
                f(x, e);
            });
        })
    }

    /// Create a callback to asynchronously mutate state with given function, provided the fired
    /// event.
    ///
    /// ```ignore
    /// let incr = dispatch.reduce_mut_future_callback_with(|state, count| Box::pin(async move {
    ///     state.count += count;
    /// }));
    /// ```
    ///
    #[cfg(feature = "future")]
    fn reduce_mut_future_callback_with<R, F, E>(&self, f: F) -> Callback<E>
    where
        S: Clone,
        F: Fn(&mut S, E) -> Pin<Box<dyn Future<Output = R> + '_>> + 'static,
        E: 'static,
    {
        let f = Rc::new(f);
        Callback::from(move |e: E| {
            let f = f.clone();
            wasm_bindgen_futures::spawn_local(async move {
                Self::new().reduce_mut_future(move |s| f(s, e)).await;
            })
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[derive(Clone, PartialEq, Eq)]
    struct TestState(u32);
    impl Store for TestState {
        fn new() -> Self {
            Self(0)
        }

        fn should_notify(&self, other: &Self) -> bool {
            self != other
        }
    }
    #[derive(PartialEq, Eq)]
    struct TestStateNoClone(u32);
    impl Store for TestStateNoClone {
        fn new() -> Self {
            Self(0)
        }

        fn should_notify(&self, other: &Self) -> bool {
            self != other
        }
    }

    struct Msg;
    impl Reducer<TestState> for Msg {
        fn apply(&self, state: Rc<TestState>) -> Rc<TestState> {
            TestState(state.0 + 1).into()
        }
    }

    #[test]
    fn dispatch_set_callback_works() {
        let dispatch = Dispatch::<TestState>::new();
        let old = dispatch.get();

        let cb = dispatch.set_callback(|_| TestState(1));
        cb.emit(());

        assert!(dispatch.get() != old)
    }

    #[test]
    fn dispatch_reduce_mut_works() {
        let dispatch = Dispatch::<TestState>::new();
        let old = dispatch.get();

        dispatch.reduce_mut(|state| state.0 += 1);

        assert!(dispatch.get() != old)
    }

    #[cfg(feature = "future")]
    #[async_std::test]
    async fn dispatch_reduce_mut_future_works() {
        let dispatch = Dispatch::<TestState>::new();
        let old = dispatch.get();

        dispatch
            .reduce_mut_future(|state| Box::pin(async move { state.0 += 1 }))
            .await;

        assert!(dispatch.get() != old)
    }

    #[test]
    fn dispatch_reduce_works() {
        let dispatch = Dispatch::<TestState>::new();
        let old = dispatch.get();

        dispatch.reduce(|_| TestState(1));

        assert!(dispatch.get() != old)
    }

    #[cfg(feature = "future")]
    #[async_std::test]
    async fn dispatch_reduce_future_works() {
        let dispatch = Dispatch::<TestState>::new();
        let old = dispatch.get();

        dispatch
            .reduce_future(|state| async move { TestState(state.0 + 1) })
            .await;

        assert!(dispatch.get() != old)
    }

    #[test]
    fn dispatch_reduce_callback_works() {
        let dispatch = Dispatch::<TestState>::new();
        let old = dispatch.get();

        let cb = dispatch.reduce_callback(|_| TestState(1));
        cb.emit(());

        assert!(dispatch.get() != old)
    }

    #[cfg(feature = "future")]
    #[async_std::test]
    async fn dispatch_reduce_future_callback_compiles() {
        let dispatch = Dispatch::<TestState>::new();

        let _ = dispatch
            .reduce_future_callback::<_, _, _, ()>(|state| async move { TestState(state.0 + 1) });
    }

    #[test]
    fn dispatch_reduce_mut_callback_works() {
        let dispatch = Dispatch::<TestState>::new();
        let old = dispatch.get();

        let cb = dispatch.reduce_mut_callback(|state| state.0 += 1);
        cb.emit(());

        assert!(dispatch.get() != old)
    }

    #[cfg(feature = "future")]
    #[async_std::test]
    async fn dispatch_reduce_mut_future_callback_compiles() {
        let dispatch = Dispatch::<TestState>::new();

        let _ = dispatch.reduce_mut_future_callback::<_, _, ()>(|state| {
            Box::pin(async move {
                state.0 += 1;
            })
        });
    }

    #[cfg(feature = "future")]
    #[async_std::test]
    async fn dispatch_reduce_future_callback_with_compiles() {
        let dispatch = Dispatch::<TestState>::new();

        let _ = dispatch
            .reduce_future_callback_with(|state, e: u32| async move { TestState(state.0 + e) });
    }

    #[test]
    fn dispatch_reduce_callback_with_works() {
        let dispatch = Dispatch::<TestState>::new();
        let old = dispatch.get();

        let cb = dispatch.reduce_callback_with(|_, _| TestState(1));
        cb.emit(1);

        assert!(dispatch.get() != old)
    }

    #[test]
    fn dispatch_reduce_mut_callback_with_works() {
        let dispatch = Dispatch::<TestState>::new();
        let old = dispatch.get();

        let cb = dispatch.reduce_mut_callback_with(|state, val| state.0 += val);
        cb.emit(1);

        assert!(dispatch.get() != old)
    }

    #[cfg(feature = "future")]
    #[async_std::test]
    async fn dispatch_reduce_mut_future_callback_with_compiles() {
        let dispatch = Dispatch::<TestState>::new();

        let _ = dispatch.reduce_mut_future_callback_with::<_, _, u32>(|state, e| {
            Box::pin(async move {
                state.0 += e;
            })
        });
    }

    #[test]
    fn dispatch_apply_works() {
        let dispatch = Dispatch::<TestState>::new();
        let old = dispatch.get();

        dispatch.apply(Msg);

        assert!(dispatch.get() != old)
    }

    #[test]
    fn dispatch_apply_callback_works() {
        let dispatch = Dispatch::<TestState>::new();
        let old = dispatch.get();

        let cb = dispatch.apply_callback(|_| Msg);
        cb.emit(());

        assert!(dispatch.get() != old)
    }
}
