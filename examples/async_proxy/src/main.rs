#![feature(async_closure)]

use yew::prelude::*;
use yewdux::prelude::*;

mod proxy;

use proxy::State;

#[function_component]
fn App() -> Html {
    let state = use_store_value::<State>();

    html! {
        <div>
        <NewTimeZone />
        <table style="width:100%">
        {
            state.timezones().map(|timezone| {
                html! { <Time timezone={timezone.clone()} /> }
            }).collect::<Html>()
        }
        </table>
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct TimeProps {
    timezone: String,
}

#[function_component]
fn Time(props: &TimeProps) -> Html {
    let dispatch = Dispatch::<State>::new();
    let timezone = props.timezone.clone();
    let refresh = dispatch.reduce_mut_callback(move |state| state.refresh(timezone.clone()));
    let timezone = props.timezone.clone();
    let delete = dispatch.reduce_mut_callback(move |state| state.delete(&timezone));

    let timezone = props.timezone.clone();
    let result = use_selector(move |state: &State| state.get(&timezone) );
    let datetime = result.0.clone();
    let status = result.1.clone();
    let timezone = props.timezone.clone();
    html! {
        <tr>
            <th> { timezone } </th>
            <td> { datetime } </td>
            <td> { format!("{:?}", status) } </td>
            <td> <button onclick={refresh}> { "Refresh" } </button> </td>
            <td> <button onclick={delete}> { "Delete" } </button> </td>
        </tr>
    }
}

#[function_component]
fn NewTimeZone() -> Html {
    let onkeypress = Callback::from(|e: KeyboardEvent| {
        if e.key() == "Enter" {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            let timezone = input.value();
            input.set_value("");
            let dispatch = Dispatch::<State>::new();
            dispatch.reduce_mut(|state| state.add(timezone.to_string()));
        }
    });

    html! {
        <input type="text" placeholder="Australia/Melbourne" {onkeypress} />
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
