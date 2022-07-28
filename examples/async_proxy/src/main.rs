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
                <tr>
                    <th width="30%"> { "Timezone" } </th>
                    <th width="40%"> { "Datetime" } </th>
                    <th width="10%"> { "Status" } </th>
                    <th width="10%"> { "Refresh" } </th>
                    <th width="10%"> { "Delete" } </th>
                </tr>
                {
                    state.timezones().iter().map(|timezone| {
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
    let result = use_selector(move |state: &State| {
        state.get(&timezone)
    });

    let timezone = props.timezone.clone();
    match &*result {
        None => { html! { <tr key={timezone.as_str()}> <td> { "Missing timezone" } </td> </tr> } },
        Some((datetime, status)) => {
            html! {
                <tr key={timezone.as_str()}>
                    <td> { timezone } </td>
                    <td> { datetime } </td>
                    <td> { format!("{:?}", status) } </td>
                    <td> <button onclick={refresh}> { "Refresh" } </button> </td>
                    <td> <button onclick={delete}> { "Delete" } </button> </td>
                </tr>
            }
        },
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

    let dispatch = Dispatch::<State>::new();
    let add_mel = dispatch.reduce_mut_callback(|state| state.add("Australia/Melbourne".to_string()));
    let add_adl = dispatch.reduce_mut_callback(|state| state.add("Australia/Adelaide".to_string()));
    let add_uto = dispatch.reduce_mut_callback(|state| state.add("Utopia".to_string()));

    html! {
        <div>
        <input type="text" placeholder="Australia/Melbourne" {onkeypress} /> { "⏎" }
        <div><button onclick={add_mel}> { "Australia/Melbourne" } </button></div>
        <div><button onclick={add_adl}> { "Australia/Adelaide" } </button></div>
        <div><button onclick={add_uto}> { "Utopia" } </button></div>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
