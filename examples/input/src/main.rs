use chrono::{DateTime, Local};
use yew::prelude::*;
use yewdux::prelude::*;
use yewdux_input::{Checkbox, InputDispatch};

#[derive(Store, Default, PartialEq, Clone)]
struct Form {
    text: String,
    checkbox: Checkbox,
    datetime: Option<DateTime<Local>>,
    // datetime: Option<String>,
    radio: Option<String>,
    range: i32,
    textarea: String,
}

#[function_component]
fn InputText() -> Html {
    let (store, dispatch) = use_store::<Form>();
    let oninput = dispatch.input(|s, text| {
        Form {
            text,
            ..s.as_ref().clone()
        }
        .into()
    });
    html! {
        <>
        <p>{&store.text}</p>
        <input {oninput} />
        </>
    }
}

#[function_component]
fn InputCheckbox() -> Html {
    let (store, dispatch) = use_store::<Form>();
    let onchange = dispatch.input_mut(|s, value| {
        s.checkbox = value;
    });

    html! {
        <>
        <p>{store.checkbox.checked()}</p>
        <input type="checkbox" {onchange} />
        </>
    }
}

#[function_component]
fn InputRadio() -> Html {
    let (store, dispatch) = use_store::<Form>();
    let onchange = dispatch.input_mut(|s, value| {
        s.radio = Some(value);
    });

    html! {
        <>
        <p>{store.radio.clone().unwrap_or_default()}</p>
        <input onchange={onchange.clone()} type="radio" id="dog" name="animal" value="cat"/ >
        <label for="cat">{ "cat" }</label><br />
        <input onchange={onchange.clone()} type="radio" id="cat" name="animal" value="dog"/ >
        <label for="dog">{"dog"}</label><br />
        </>
    }
}

#[function_component]
fn InputDatetime() -> Html {
    let (store, dispatch) = use_store::<Form>();
    let oninput = dispatch.input_mut(|s, value| {
        s.datetime = Some(value);
    });

    html! {
        <>
        <p>{store.datetime.unwrap_or_default().to_rfc2822()}</p>
        <input type="datetime-local" {oninput} />
        </>
    }
}

#[function_component]
fn InputTextArea() -> Html {
    let (state, dispatch) = use_store::<Form>();
    let oninput = dispatch.input_mut(|s, value| {
        s.textarea = value;
    });

    html! {
        <>
        <p>{ &state.textarea }</p>
        <textarea {oninput} />
        </>
    }
}

#[function_component]
fn App() -> Html {
    html! {
        <>
        <InputText />
        <InputCheckbox />
        <InputRadio />
        <InputDatetime />
        <InputTextArea />
        </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
