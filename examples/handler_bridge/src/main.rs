use std::rc::Rc;
use yew::agent::{Bridge, Bridged, HandlerId};
use yew::prelude::*;
use yew_state::{
    handler::{Changed, HandlerLink, StateHandler},
    service::{ServiceInput, ServiceOutput, ServiceRequest, ServiceResponse, StateService},
};

#[derive(Clone)]
struct State {
    count: u32,
}

enum CountMsg {
    Increment,
}

enum Input {
    GetCountDoubled,
    Reset,
}

enum Output {
    CountDoubled(u32),
}

struct CountHandler {
    link: HandlerLink<Self>,
    state: Rc<State>,
}

impl StateHandler for CountHandler {
    type Model = State;
    type Message = CountMsg;
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

    fn update(&mut self, msg: Self::Message) -> Changed {
        match msg {
            CountMsg::Increment => {
                // Increment out count.
                let state = Rc::make_mut(&mut self.state);
                state.count += 1;
                // Tell StateService to notify subscribers of new state.
                true
            }
        }
    }

    fn handle_input(&mut self, msg: Self::Input, who: HandlerId) -> Changed {
        match msg {
            Input::GetCountDoubled => {
                self.link
                    .respond(who, Output::CountDoubled(self.state.count * 2));

                false
            }
            Input::Reset => {
                // Reset our state to 0.
                let state = Rc::make_mut(&mut self.state);
                state.count = 0;
                // Tell StateService to notify subscribers of new state.
                true
            }
        }
    }
}

enum Msg {
    /// Receive new state from StateService. This is called every time state is changed.
    SetState(Rc<State>),
    /// Receive our HandlerLink.
    SetLink(HandlerLink<CountHandler>),
    /// Initiate counter double.
    Double,
    /// Double the count.
    Doubled(u32),
    /// Reset count to 0.
    Reset,
}

struct App {
    state: Option<Rc<State>>,
    bridge: Box<dyn Bridge<StateService<CountHandler>>>,
    handler_link: Option<HandlerLink<CountHandler>>,
    link: ComponentLink<Self>,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_handle: Self::Properties, link: ComponentLink<Self>) -> Self {
        // Create a bridge to communicate with CountHandle, no component wrapper required!
        // In addition to CountHandler::Ouput, we can also handle messages sent by StateService.
        let bridge = StateService::<CountHandler>::bridge(link.callback(|msg| match msg {
            // Messages sent by our CountHandler.
            ServiceOutput::Handler(msg) => match msg {
                Output::CountDoubled(n) => Msg::Doubled(n),
            },
            // Messages sent by StateService.
            ServiceOutput::Service(msg) => match msg {
                // This is sent every time state changes.
                ServiceResponse::State(state) => Msg::SetState(state),
                // This is sent once, when first connecting. Handy if you want access to all the
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
            // Use calculated count to set new state.
            // Send update message to StateService, not CountHandler.
            Msg::Doubled(count) => {
                // Function that mutates state.
                let f = Box::new(move |state: &mut State| state.count = count);
                // Message for StateService.
                let msg = ServiceInput::Service(ServiceRequest::ApplyOnce(f));
                // Send it!.
                self.bridge.send(msg);

                false
            }
            Msg::Double => {
                // Ask CountHandler to send us count doubled.
                self.bridge
                    .send(ServiceInput::Handler(Input::GetCountDoubled));

                false
            }
            Msg::Reset => {
                // Lets use out link handle this time. Brige method is shown above.
                if let Some(link) = &self.handler_link {
                    link.send_input(Input::Reset);
                }

                false
            }
        }
    }

    fn change(&mut self, _handle: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        // View out local state.
        let view_count = self.state.as_ref().map(|s| html! { <h1>{ s.count }</h1> });
        // We can use self.handler_link to create CountHandler callbacks.
        let increment = self
            .handler_link
            .as_ref()
            .map(|l| l.callback(|_| CountMsg::Increment))
            .unwrap_or_default();
        let double = self.link.callback(|_| Msg::Double);
        let reset = self.link.callback(|_| Msg::Reset);
        html! {
            <>
            { for view_count }
            <button onclick=increment>{"Increment"}</button>
            <button onclick=double>{"Double"}</button>
            <button onclick=reset>{"Reset"}</button>
            </>
        }
    }
}

fn main() {
    yew::start_app::<App>();
}
