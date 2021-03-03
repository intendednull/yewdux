use std::rc::Rc;

use yew::agent::HandlerId;
use yew::prelude::*;
use yewdux::{
    store::{ShouldNotify, Store, StoreLink},
    Dispatch,
};

#[derive(Clone)]
struct State {
    count: u32,
}

/// Messages sent through the bridge.
enum Input {
    Reset,
    Increment,
}

struct CounterStore {
    state: Rc<State>,
}

impl Store for CounterStore {
    type Model = State;
    type Message = ();
    type Input = Input;
    type Output = ();

    fn new(_link: StoreLink<Self>) -> Self {
        Self {
            state: Rc::new(State { count: 0 }),
        }
    }

    fn state(&mut self) -> &mut Rc<Self::Model> {
        &mut self.state
    }

    fn handle_input(&mut self, msg: Self::Input, _who: HandlerId) -> ShouldNotify {
        let state = Rc::make_mut(&mut self.state);
        match msg {
            Input::Increment => {
                // Increment count by 1.
                state.count += 1;
            }
            Input::Reset => {
                // Reset our state to 0.
                state.count = 0;
            }
        };

        // Tell our parent service that state changed. This will notify all subscribers of these
        // changes.
        true
    }
}

enum Msg {
    /// Receive new state from StateService. This is called every time state is changed.
    State(Rc<State>),
}

struct App {
    dispatch: Dispatch<CounterStore>,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_handle: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            dispatch: Dispatch::new(link.callback(Msg::State)),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::State(state) => {
                self.dispatch.state = Some(state);
                true
            }
        }
    }

    fn change(&mut self, _handle: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        // We only want to display when we've received our state and link.
        if self.dispatch.is_ready() {
            let reset = self.dispatch.callback(|_| Input::Reset);
            let incr = self.dispatch.callback(|_| Input::Increment);
            let count = self.dispatch.state().count;

            html! {
                <>
                <h1>{ count }</h1>
                <button onclick=incr>{"Increment"}</button>
                <button onclick=reset>{"Reset"}</button>
                </>
            }
        } else {
            Default::default()
        }
    }
}

fn main() {
    yew::start_app::<App>();
}
