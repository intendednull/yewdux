use std::rc::Rc;

use yew::prelude::*;
use yewdux::prelude::*;

#[derive(Default, Clone)]
struct State {
    count: u32,
}

struct App {
    /// Our local version of state.
    state: Rc<State>,
    dispatch: Dispatch<BasicStore<State>>,
}

enum Msg {
    /// Message to receive new state.
    State(Rc<State>),
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            dispatch: Dispatch::bridge_state(link.callback(Msg::State)),
            state: Default::default(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::State(state) => {
                self.state = state;
                true
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let count = self.state.count;
        let onclick = self
            .dispatch
            .reduce_future_callback(|_| async { 1 }, |s, result| s.count += result);
        html! {
            <>
            <h1>{ count }</h1>
            <button onclick={onclick}>{"+1"}</button>
            </>
        }
    }
}

pub fn main() {
    yew::start_app::<App>();
}
