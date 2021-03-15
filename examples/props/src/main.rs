use yew::prelude::*;
use yewdux::prelude::*;
use yewtil::NeqAssign;

#[derive(Default, Clone)]
struct State {
    count: u32,
}

struct App {
    dispatch: DispatchProps<BasicStore<State>>,
}

impl Component for App {
    type Message = ();
    type Properties = DispatchProps<BasicStore<State>>;

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
        let onclick = self.dispatch.reduce_callback(|s| s.count += 1);
        html! {
            <>
            <h1>{ count }</h1>
            <button onclick=onclick>{"+1"}</button>
            </>
        }
    }
}

pub fn main() {
    yew::start_app::<WithDispatch<App>>();
}
