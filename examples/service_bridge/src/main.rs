use std::rc::Rc;
use yew::agent::HandlerId;
use yew::prelude::*;
use yew_state::{
    handler::{Changed, HandlerLink, StateHandler},
    service::{ServiceBridge, ServiceOutput, ServiceResponse},
};

#[derive(Clone)]
struct State {
    count: u32,
}

/// Messages sent through the HandlerLink.
enum StateMsg {
    Increment,
}

/// Messages sent through the bridge.
enum Input {
    Reset,
}

struct CountHandler {
    state: Rc<State>,
}

impl StateHandler for CountHandler {
    type Model = State;
    type Message = StateMsg;
    type Input = Input;
    type Output = ();

    fn new(_link: HandlerLink<Self>) -> Self {
        Self {
            state: Rc::new(State { count: 0 }),
        }
    }

    fn state(&mut self) -> &mut Rc<Self::Model> {
        &mut self.state
    }

    fn update(&mut self, msg: Self::Message) -> Changed {
        let state = Rc::make_mut(&mut self.state);
        match msg {
            StateMsg::Increment => {
                // Increment count by 1.
                state.count += 1;
            }
        };

        // Tell our parent service that state changed. This will notify all subscribers of these
        // changes.
        true
    }

    fn handle_input(&mut self, msg: Self::Input, _who: HandlerId) -> Changed {
        let state = Rc::make_mut(&mut self.state);
        match msg {
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
    SetState(Rc<State>),
    /// Receive our HandlerLink.
    SetLink(HandlerLink<CountHandler>),
    /// Message to reset count.
    Reset,
}

struct App {
    state: Option<Rc<State>>,
    #[allow(dead_code)]
    bridge: ServiceBridge<CountHandler>,
    handler_link: Option<HandlerLink<CountHandler>>,
    link: ComponentLink<Self>,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_handle: Self::Properties, link: ComponentLink<Self>) -> Self {
        // Create a bridge to communicate with CountHandle, no component wrapper required!
        // We'll use this to receive messages that SharedStateComponent usually handles for us.
        let bridge = ServiceBridge::new(link.callback(|msg| match msg {
            // This is where we would receive our handler output message if we had any.
            ServiceOutput::Handler(msg) => match msg {
                _ => unreachable!(),
            },
            // Messages sent by StateService.
            ServiceOutput::Service(msg) => match msg {
                // This is received every time state changes.
                ServiceResponse::State(state) => Msg::SetState(state),
                // This is received once, when first connecting. Handy if you want access to all the
                // handler link methods.
                ServiceResponse::Link(link) => Msg::SetLink(link),
            },
        }));

        Self {
            bridge,
            link,
            handler_link: Default::default(),
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
            // Receive HandlerLink.
            Msg::SetLink(link) => {
                self.handler_link = Some(link);
                true
            }
            // Send message to reset count. We *could* have done this through our HandlerLink,
            // but then we wouldn't get to see bridge message passing.
            Msg::Reset => {
                self.bridge.send_handler(Input::Reset);
                false
            }
        }
    }

    fn change(&mut self, _handle: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        // We only want to display when we've received our state and link.
        if let (Some(state), Some(link)) = (&self.state, &self.handler_link) {
            let reset = self.link.callback(|_| Msg::Reset);
            let incr = link.callback(|_| StateMsg::Increment);

            html! {
                <>
                <h1>{ state.count }</h1>
                <button onclick=incr>{"Increment"}</button>
                <button onclick=reset>{"Reset"}</button>
                </>
            }
        } else {
            html! {}
        }
    }
}

fn main() {
    yew::start_app::<App>();
}
