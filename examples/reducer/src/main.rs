use yew::prelude::*;
use yewdux::prelude::*;

enum Action {
    Increment,
}

#[derive(Clone)]
struct Counter {
    count: u64,
}

impl Reducer for Counter {
    type Action = Action;

    fn new() -> Self {
        Self { count: 0 }
    }

    fn reduce(&mut self, action: Self::Action) -> Changed {
        match action {
            Action::Increment => {
                self.count += 1;
                true
            }
        }
    }
}

type AppDispatch = DispatchProps<ReducerStore<Counter>>;

struct App;

impl Component for App {
    type Message = ();
    type Properties = AppDispatch;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let count = ctx.props().state().count;
        let increment = ctx.props().callback(|_| Action::Increment);
        html! {
            <>
            <h1>{ count }</h1>
            <button onclick={increment}>{"+1"}</button>
            </>
        }
    }
}

fn main() {
    yew::start_app::<WithDispatch<App>>();
}
