#![cfg(target_arch = "wasm32")]

use yew::prelude::*;
use yewdux::prelude::*;

mod proxy;

use proxy::State;

#[function_component]
fn App() -> Html {
    let state = use_store_value::<State>();
    let timezones = state
        .timezones()
        .map(|timezone| {
            html! { <Time {timezone} /> }
        })
        .collect::<Html>();

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
                { timezones }
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
    let dispatch = Dispatch::<State>::global();
    let timezone = props.timezone.clone();
    let refresh = {
        let timezone = timezone.clone();
        dispatch.reduce_mut_callback(move |state| state.refresh(timezone.clone()))
    };
    let delete = {
        let timezone = timezone.clone();
        dispatch.reduce_mut_callback(move |state| state.delete(&timezone))
    };
    let result = {
        let timezone = timezone.clone();
        use_selector_with_deps(|state: &State, timezone| state.get(timezone), timezone)
    };
    let content = match result.as_ref() {
        None => {
            html! { <td>{ "Missing timezone" }</td> }
        }
        Some((datetime, status)) => {
            html! {
                <>
                <td> { &timezone } </td>
                <td> { datetime } </td>
                <td> { format!("{:?}", status) } </td>
                <td> <button onclick={refresh}> { "Refresh" } </button> </td>
                <td> <button onclick={delete}> { "Delete" } </button> </td>
                </>
            }
        }
    };

    html! {
        <tr key={timezone}>
            { content }
        </tr>
    }
}

#[function_component]
fn NewTimeZone() -> Html {
    let dispatch = Dispatch::<State>::global();
    let onkeypress = dispatch.reduce_mut_callback_with(|state, e: KeyboardEvent| {
        if e.key() == "Enter" {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            let timezone = input.value();
            input.set_value("");

            state.add(timezone)
        }
    });
    let add_mel =
        dispatch.reduce_mut_callback(|state| state.add("Australia/Melbourne".to_string()));
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
