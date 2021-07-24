use yew::{functional::*, prelude::*};
use yewdux::prelude::*;
use yewdux_functional::use_store;
use yewdux_input::*;

#[derive(Default, Clone)]
struct UserForm {
    first: String,
    last: String,
    pet: String,
    avatar: String,
}

#[function_component(InputName)]
fn input_name() -> Html {
    let form = use_store::<BasicStore<UserForm>>();
    let input_first = {
        let value = form.state().map(|s| s.first.clone()).unwrap_or_default();
        let oninput = form.dispatch().input(|form, value| form.first = value);
        html! {
            <input placeholder="First name" value={value} oninput={oninput} />
        }
    };
    let input_last = {
        let value = form.state().map(|s| s.last.clone()).unwrap_or_default();
        let oninput = form.dispatch().input(|form, value| form.last = value);
        html! {
            <input placeholder="Last name" value={value} oninput={oninput} />
        }
    };

    html! {
        <>
        { input_first }
        { input_last }
        </>
    }
}

#[function_component(InputPet)]
fn input_pet() -> Html {
    let form = use_store::<BasicStore<UserForm>>();
    let onchange = form.dispatch().select(|form, value| form.pet = value);
    let value = form.state().map(|s| s.pet.clone()).unwrap_or_default();
    let options = ["None", "Cat", "Dog"]
        .iter()
        .map(|&val| {
            let selected = value == val;
            html! {
                <option selected={selected} value={val}>{ val }</option>
            }
        })
        .collect::<Html>();
    html! {
        <>
        <label>{"Do you have a pet?"}</label>
        {" "}
        <select onchange={onchange}>
            { options }
        </select>
        </>
    }
}

// #[function_component(InputAvatar)]
// fn input_avatar() -> Html {
// let form = use_store::<BasicStore<UserForm>>();
// let onchange = form.dispatch().file(|form, value| form.avatar = value.name);
// html! {
// <>
// <label>{"Select an avatar"}</label>
// {" "}
// <input type="file" onchange={onchange} />
// </>
// }
// }

#[function_component(App)]
fn app() -> Html {
    let form = use_store::<BasicStore<UserForm>>();
    let user = form
        .state()
        .map(|s| {
            html! {
                <>
                <h1>{ &s.first }{" "}{ &s.last }</h1>
                <h2>{"Pet: "}{ &s.pet }</h2>
                <h2>{"Avatar: "}{ &s.avatar }</h2>
                </>
            }
        })
        .unwrap_or_default();

    html! {
        <>
        { user }
        <div>
            <div><InputName /></div>
            <div><InputPet /></div>
            // <div><InputAvatar /></div>
        </div>
        </>

    }
}

pub fn main() {
    yew::start_app::<App>();
}
