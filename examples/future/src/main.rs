use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use yew::prelude::*;
use yewdux::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cat {
    url: String,
}

async fn fetch_cats(limit: u32) -> Result<Cats, ()> {
    let cats = Request::get(&format!(
        "https://api.thecatapi.com/v1/images/search?limit={}",
        limit
    ))
    .send()
    .await
    .map_err(|_| ())?
    .json::<Vec<Cat>>()
    .await
    .map_err(|_| ())?
    .into_iter()
    .collect::<Vec<_>>();

    Ok(Cats(cats))
}

#[derive(Default, Clone, PartialEq, Eq, Store)]
struct Cats(Vec<Cat>);

#[function_component]
fn App() -> Html {
    let (state, dispatch) = use_store::<Cats>();

    let fetch_single = dispatch.future_callback(|dispatch| async move {
        let result = fetch_cats(5).await;
        if let Ok(cats) = result {
            dispatch.set(cats);
        }
    });

    let cats = state
        .0
        .iter()
        .map(|Cat { url }| {
            html! {
                <img src={url.clone()} />
            }
        })
        .collect::<Html>();

    html! {
        <>
        <button onclick={fetch_single}>{"meow"}</button>
        <div>
            { cats }
        </div>
        </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
