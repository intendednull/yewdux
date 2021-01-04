use std::rc::Rc;
use yew::prelude::*;
use yew_state::{
    handler::{Changed, HandlerLink, StateHandler},
    LinkHandle, SharedStateComponent, StateHandle,
};
use yewtil::NeqAssign;

#[derive(Clone, PartialEq)]
struct State {
    count: u32,
}

enum CountMsg {
    Increment,
}

struct CountHandler {
    link: HandlerLink<CountMsg>,
    state: Rc<State>,
}

impl StateHandler for CountHandler {
    type Model = State;
    type Message = CountMsg;

    fn new(link: HandlerLink<Self::Message>) -> Self {
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
                let state = Rc::make_mut(&mut self.state);
                state.count += 1;

                true
            }
        }
    }
}

type Handle = LinkHandle<CountHandler>;

struct Model {
    handle: Handle,
}

impl Component for Model {
    type Message = ();
    type Properties = Handle;

    fn create(handle: Self::Properties, _link: ComponentLink<Self>) -> Self {
        handle.link().send_message(CountMsg::Increment);
        Self { handle }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, handle: Self::Properties) -> ShouldRender {
        self.handle.neq_assign(handle)
    }

    fn view(&self) -> Html {
        html! { self.handle.state().count }
    }
}

type App = SharedStateComponent<Model>;

fn main() {
    yew::start_app::<App>();
}
