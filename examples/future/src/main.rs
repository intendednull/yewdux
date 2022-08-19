use yew::prelude::*;
use yewdux::prelude::*;

#[derive(Default, Clone, PartialEq, Eq, Store)]
struct State {
    count: u32,
}

#[function_component(App)]
fn app() -> Html {
    let (state, dispatch) = use_store::<State>();

    let incr = dispatch.reduce_future_callback(|state| async move {
        State {
            count: state.count + 1,
        }
    });
    let incr_mut = dispatch.reduce_mut_future_callback(|state| {
        Box::pin(async move {
            state.count += 1;
        })
    });

    html! {
        <>
        <p>{ state.count }</p>
        <button onclick={incr}>{"+1 reduce"}</button>
        <button onclick={incr_mut}>{"+1 reduce_mut"}</button>
        </>
    }
}

fn main() {
    yew::start_app::<App>();
}
