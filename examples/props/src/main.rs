use yew::prelude::*;
use yewdux::prelude::*;

#[derive(Default, Clone)]
struct State {
    count: u32,
}

struct App;

impl Component for App {
    type Message = ();
    type Properties = DispatchProps<BasicStore<State>>;

    fn create(ctx: &Context<Self>) -> Self {
        Self
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let count = ctx.props().state().count;
        let onclick = ctx.props().reduce_callback(|s| s.count += 1);
        html! {
            <>
            <h1>{ count }</h1>
            <button onclick={onclick}>{"+1"}</button>
            </>
        }
    }
}

pub fn main() {
    yew::start_app::<WithDispatch<App>>();
}
