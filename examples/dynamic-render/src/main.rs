use yew::prelude::*;
use yewdux::prelude::*;

#[function_component]
fn App() -> Html {
    let (points, dispatch) = use_store::<Points>();
    let cb = dispatch.reduce_callback(|points| {
        Points {
            points: points.points + 1,
        }
        .into()
    });
    let view = (0..points.points)
        .map(|_| {
            html! {
                <div>
                    <Listener/>
                </div>
            }
        })
        .collect::<Vec<_>>();

    html! {
        <div>
            <button onclick={cb}>{"Click me"}</button>
            {view}
        </div>
    }
}

#[function_component]
fn Listener() -> Html {
    let points = use_store_value::<Points>();
    html! { points.points }
}

#[derive(Store, PartialEq, Eq, Default, Clone)]
struct Points {
    points: usize,
}

fn main() {
    yew::Renderer::<App>::new().render();
}
