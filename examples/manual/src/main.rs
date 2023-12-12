#![cfg(target_arch = "wasm32")]

use std::rc::Rc;

use yew::prelude::*;
use yewdux::prelude::*;

#[derive(Default, Clone, PartialEq, Eq, Store)]
struct State {
    count: u32,
}

struct App {
    /// Our local version of state.
    state: Rc<State>,
    dispatch: Dispatch<State>,
}

enum Msg {
    /// Message to receive new state.
    State(Rc<State>),
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let dispatch = Dispatch::<State>::global().subscribe(ctx.link().callback(Msg::State));
        Self {
            state: dispatch.get(),
            dispatch,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::State(state) => {
                self.state = state;
                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let count = self.state.count;
        let onclick = self.dispatch.reduce_mut_callback(|s| s.count += 1);
        html! {
            <>
            <h1>{ count }</h1>
            <button onclick={onclick}>{"+1"}</button>
            </>
        }
    }
}

pub fn main() {
    yew::Renderer::<App>::new().render();
}
