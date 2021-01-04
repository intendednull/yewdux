use std::rc::Rc;
use yew::agent::HandlerId;
use yew::prelude::*;
use yew_state::handler::{Changed, HandlerBridge, HandlerLink, StateHandler};

#[derive(Clone)]
struct State {
    count: u32,
}

enum Input {
    Reset,
    Increment,
}

enum Output {
    State(Rc<State>),
}

struct CountHandler {
    link: HandlerLink<Self>,
    state: Rc<State>,
}

impl StateHandler for CountHandler {
    type Model = State;
    type Message = ();
    type Input = Input;
    type Output = Output;

    fn new(link: HandlerLink<Self>) -> Self {
        Self {
            link,
            state: Rc::new(State { count: 0 }),
        }
    }

    fn state(&mut self) -> &mut Rc<Self::Model> {
        &mut self.state
    }

    fn handle_input(&mut self, msg: Self::Input, who: HandlerId) -> Changed {
        let state = Rc::make_mut(&mut self.state);
        match msg {
            Input::Reset => {
                // Reset our state to 0.
                state.count = 0;
            }
            Input::Increment => {
                // Increment count by 1.
                state.count += 1;
            }
        };

        // Respond with new state.
        self.link
            .respond(who, Output::State(Rc::clone(&self.state)));

        // Tell our parent service that state changed. This will notify all subscribers of these
        // changes.
        true
    }
}

enum Msg {
    /// Receive new state.
    SetState(Rc<State>),
    /// Input message for CountHandler.
    Input(Input),
}

struct App {
    state: Option<Rc<State>>,
    bridge: HandlerBridge<CountHandler>,
    link: ComponentLink<Self>,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_handle: Self::Properties, link: ComponentLink<Self>) -> Self {
        // Receive handler output.
        let bridge = HandlerBridge::new(link.callback(|msg| match msg {
            Output::State(s) => Msg::SetState(s),
        }));

        Self {
            bridge,
            link,
            state: Default::default(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            // Receive new state.
            Msg::SetState(state) => {
                self.state = Some(state);
                true
            }
            // Send input message to handler.
            Msg::Input(msg) => {
                self.bridge.send(msg);
                false
            }
        }
    }

    fn change(&mut self, _handle: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        // The current count.
        let count = self.state.as_ref().map(|s| s.count);
        // Callbacks for modifying count.
        let increment = self.link.callback(|_| Msg::Input(Input::Increment));
        let reset = self.link.callback(|_| Msg::Input(Input::Reset));

        html! {
            <>
            <h1>{ for count }</h1>
            <button onclick=increment>{"Increment"}</button>
            <button onclick=reset>{"Reset"}</button>
            </>
        }
    }
}

fn main() {
    yew::start_app::<App>();
}
