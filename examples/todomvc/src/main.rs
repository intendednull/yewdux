#![cfg(target_arch = "wasm32")]

mod state;

use web_sys::HtmlInputElement;
use yew::prelude::*;
use yewdux::prelude::*;

use state::{Entry, Filter, State};

#[function_component]
fn App() -> Html {
    let hidden = use_selector(|s: &State| s.entries.is_empty());
    let hidden_class = if *hidden { "hidden" } else { "" };

    html! {
        <div class="todomvc-wrapper">
            <section class="todoapp">
                <Header />
               <section class={classes!("main", hidden_class)} >
                    <ToggleAll />
                    <ListEntries />
                </section>
                <footer class={classes!("footer", hidden_class)}>
                    <Footer />
                </footer>
            </section>
            <footer class="info">
                <p>{ "Double-click to edit a todo" }</p>
                <p>{ "Written by " }<a href="https://github.com/intendednull" target="_blank">
                    { "Noah Corona" }
                </a></p>
                <p>{ "Part of " }<a href="http://todomvc.com/" target="_blank">{ "TodoMVC" }</a></p>
            </footer>
        </div>
    }
}

#[function_component]
fn Header() -> Html {
    let onkeypress = Dispatch::<State>::global().reduce_mut_callback_with(|s, e: KeyboardEvent| {
        if e.key() == "Enter" {
            let input: HtmlInputElement = e.target_unchecked_into();
            let value = input.value();
            input.set_value("");
            if !value.is_empty() {
                let entry = Entry {
                    description: value.trim().to_string(),
                    completed: false,
                    editing: false,
                };
                s.entries.push(entry);
            }
        }
    });

    html! {
        <header class="header">
            <h1>{ "todos" }</h1>
            <input
                class="new-todo"
                placeholder="What needs to be done?"
                {onkeypress}
                />
        </header>
    }
}

#[derive(Properties, PartialEq, Clone)]
struct SelectFilterProps {
    active: Filter,
    target: Filter,
}

#[function_component]
fn SelectFilter(&SelectFilterProps { active, target }: &SelectFilterProps) -> Html {
    let cls = if active == target {
        "selected"
    } else {
        "not-selected"
    };
    let onclick = Dispatch::<State>::global().reduce_mut_callback(move |s| s.filter = target);
    html! {
        <li>
            <a class={cls} href={target.as_href()} {onclick}>
                { target.to_string() }
            </a>
        </li>
    }
}

#[function_component]
fn Footer() -> Html {
    let active_filter = use_selector(|s: &State| s.filter);
    let filters = [Filter::All, Filter::Active, Filter::Completed]
        .iter()
        .copied()
        .map(|target| html! { <SelectFilter active={*active_filter} {target} /> })
        .collect::<Html>();

    html! {
        <>
        <TodoCount />
        <ul class="filters">
            { filters }
        </ul>
        <BtnClearCompleted />
        </>
    }
}

#[function_component]
fn BtnClearCompleted() -> Html {
    let total_completed = use_selector(|state: &State| state.total_completed());
    let onclick = Dispatch::<State>::global().reduce_mut_callback(|s| s.clear_completed());

    html! {
        <button class="clear-completed" {onclick} >
            { format!("Clear completed ({})", total_completed) }
        </button>
    }
}

#[function_component]
fn TodoCount() -> Html {
    let count = use_selector(|state: &State| state.total());

    html! {
        <span class="todo-count">
            <strong>{ *count }</strong>
            { " item(s) left" }
        </span>
    }
}

#[function_component]
fn ToggleAll() -> Html {
    let checked = use_selector(|state: &State| state.is_all_completed());
    let onclick = Dispatch::global().reduce_mut_callback(|s: &mut State| {
        let status = !s.is_all_completed();
        s.toggle_all(status);
    });

    html! {
        <>
        <input type="checkbox" class="toggle-all" id="toggle-all" checked={*checked} {onclick} />
        <label for="toggle-all" />
        </>
    }
}

#[derive(Properties, PartialEq, Clone)]
struct EntryIdProps {
    id: usize,
}

#[function_component]
fn EntryToggle(&EntryIdProps { id }: &EntryIdProps) -> Html {
    let (state, dispatch) = use_store::<State>();
    let entry = &state.entries[id];
    let onclick = dispatch.reduce_mut_callback(move |s| s.toggle(id));
    html! {
        <input
            type="checkbox"
            class="toggle"
            checked={entry.completed}
            {onclick}
        />
    }
}

#[function_component]
fn EntryDesc(&EntryIdProps { id }: &EntryIdProps) -> Html {
    let (state, dispatch) = use_store::<State>();
    let entry = &state.entries[id];
    let ondblclick = dispatch.reduce_mut_callback(move |s| {
        s.edit_value = s.entries[id].description.clone();
        s.clear_all_edit();
        s.toggle_edit(id);
    });
    let onclick = dispatch.reduce_mut_callback(move |s| s.remove(id));
    html! {
        <>
        <label {ondblclick}>{ &entry.description }</label>
        <button class="destroy" {onclick} />
        </>
    }
}

#[function_component]
fn EntryEdit(&EntryIdProps { id }: &EntryIdProps) -> Html {
    let focus_ref = use_node_ref();
    let (state, dispatch) = use_store::<State>();
    let entry = &state.entries[id];
    let edit = move |input: HtmlInputElement, state: &mut State| {
        let value = input.value();
        input.set_value("");

        state.complete_edit(id, value.trim().to_string());
        state.edit_value = "".to_string();
    };
    let onblur = dispatch
        .reduce_mut_callback_with(move |s, e: FocusEvent| edit(e.target_unchecked_into(), s));
    let onmouseover = {
        let focus_ref = focus_ref.clone();
        Callback::from(move |_| {
            if let Some(input) = focus_ref.cast::<HtmlInputElement>() {
                input.focus().unwrap();
            }
        })
    };
    let onkeypress = dispatch.reduce_mut_callback_with(move |s, e: KeyboardEvent| {
        if e.key() == "Enter" {
            edit(e.target_unchecked_into(), s)
        }
    });

    if entry.editing {
        html! {
            <input
                class="edit"
                type="text"
                ref={focus_ref.clone()}
                value={state.edit_value.clone()}
                {onmouseover}
                {onblur}
                {onkeypress}
            />
        }
    } else {
        html! { <input type="hidden" /> }
    }
}

#[function_component]
fn ViewEntry(&EntryIdProps { id }: &EntryIdProps) -> Html {
    let mut class = Classes::from("todo");
    let state = use_store_value::<State>();
    let entry = &state.entries[id];

    if entry.editing {
        class.push(" editing");
    }
    if entry.completed {
        class.push(" completed");
    }

    html! {
        <li {class}>
            <div class="view">
                <EntryToggle {id} />
                <EntryDesc {id} />
            </div>
            <EntryEdit {id} />
        </li>
    }
}

#[function_component]
fn ListEntries() -> Html {
    let state = use_store_value::<State>();
    let entries = state
        .entries
        .iter()
        .enumerate()
        .filter(|(_, e)| state.filter.fits(e))
        .map(|(id, _)| html! { <ViewEntry {id} /> })
        .collect::<Html>();

    html! {
        <ul class="todo-list">
            { entries }
        </ul>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
