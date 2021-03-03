use yew::prelude::*;
use yewdux::{Dispatch, Reducer, ReducerStore, ShouldNotify, WithDispatch};
use yewtil::NeqAssign;

enum Action {
    Increment,
}

#[derive(Clone, PartialEq)]
struct Counter {
    count: u64,
}

impl Reducer for Counter {
    type Action = Action;

    fn new() -> Self {
        Self { count: 0 }
    }

    fn reduce(&mut self, action: Self::Action) -> ShouldNotify {
        match action {
            Action::Increment => {
                self.count += 1;
                true
            }
        }
    }
}

type AppDispatch = Dispatch<ReducerStore<Counter>>;

struct App {
    dispatch: AppDispatch,
}

impl Component for App {
    type Message = ();
    type Properties = AppDispatch;

    fn create(dispatch: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self { dispatch }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, dispatch: Self::Properties) -> ShouldRender {
        self.dispatch.neq_assign(dispatch)
    }

    fn view(&self) -> Html {
        let count = self.dispatch.state().count;
        let increment = self.dispatch.callback(|_| Action::Increment);
        html! {
            <>
            <p>{ count }</p>
            <button onclick=increment>{"+1"}</button>
            </>
        }
    }
}

fn main() {
    yew::start_app::<WithDispatch<App>>();
}
