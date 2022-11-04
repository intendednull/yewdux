use std::rc::Rc;

use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yewdux::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cat {
    url: String,
}

async fn fetch_cats(limit: u32) -> Result<Vec<Cat>, ()> {
    let res = Request::get(&format!(
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

    Ok(res)
}

#[derive(Default, Clone, PartialEq, Eq, Store)]
struct Cats {
    inner: Vec<Cat>,
}

enum FetchCats {
    Limit(u32),
    Single,
}

#[async_reducer]
impl AsyncReducer<Cats> for FetchCats {
    async fn apply(self, mut cats: Rc<Cats>) -> Rc<Cats> {
        let limit = match self {
            FetchCats::Limit(limit) => limit,
            FetchCats::Single => 1,
        };

        if let Ok(val) = fetch_cats(limit).await {
            let state = Rc::make_mut(&mut cats);
            state.inner.extend(val);
        }

        cats
    }
}

#[function_component]
fn App() -> Html {
    let (state, dispatch) = use_store::<Cats>();
    let fetch_unlimited = dispatch.apply_future_callback(|_| FetchCats::Single);
    let fetch_limited = dispatch.apply_future_callback(|e: Event| {
        let value = e
            .target_unchecked_into::<HtmlInputElement>()
            .value()
            .parse()
            .unwrap_or_default();

        FetchCats::Limit(value)
    });

    let cats = state
        .inner
        .iter()
        .map(|Cat { url }| {
            html! {
                <img src={url.clone()} />
            }
        })
        .collect::<Html>();

    html! {
        <>
        <button onclick={fetch_unlimited}>{"Fetch a cat"}</button>
        <input type="number"
            onchange={fetch_limited} placeholder="Fetch many cats (enter a number)" />
        <div>
            { cats }
        </div>
        </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
