use std::rc::Rc;
use yew::agent::HandlerId;
use yew::prelude::*;
use yewdux::{
    store::{ShouldNotify, Store, StoreLink},
    Dispatch, WithDispatch,
};
use yewtil::NeqAssign;

#[derive(Clone, PartialEq)]
struct State {
    count: u32,
}

#[derive(Clone, PartialEq)]
enum CountMsg {
    Increment,
}

struct CounterStore {
    state: Rc<State>,
}

impl Store for CounterStore {
    type Model = State;
    type Message = ();
    type Input = CountMsg;
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
        match msg {
            CountMsg::Increment => {
                let state = Rc::make_mut(&mut self.state);
                state.count += 1;

                true
            }
        }
    }
}

struct Model {
    dispatch: Dispatch<CounterStore>,
}

impl Component for Model {
    type Message = ();
    type Properties = Dispatch<CounterStore>;

    fn create(dispatch: Self::Properties, _link: ComponentLink<Self>) -> Self {
        dispatch.send(CountMsg::Increment);
        Self { dispatch }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, handle: Self::Properties) -> ShouldRender {
        self.dispatch.neq_assign(handle)
    }

    fn view(&self) -> Html {
        html! { self.dispatch.state().count }
    }
}

type App = WithDispatch<Model>;

fn main() {
    yew::start_app::<App>();
}
