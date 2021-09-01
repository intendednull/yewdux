use yew::prelude::*;
use yewdux::prelude::*;

#[derive(Default, Clone)]
struct State {
    count: u32,
}

#[derive(Clone, PartialEq, Properties, Default)]
struct Props {
    dispatch: DispatchProps<BasicStore<State>>,
}

impl Dispatched for Props {
    type Store = BasicStore<State>;

    fn dispatch(&self) -> &DispatchProps<Self::Store> {
        &self.dispatch
    }
}

struct App;

impl Component for App {
    type Message = ();
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        Self
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let count = ctx.props().dispatch.state().count;
        let onclick = ctx.props().dispatch.reduce_callback(|s| s.count += 1);
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
