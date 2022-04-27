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
    let onkeypress = Dispatch::<State>::new().reduce_mut_callback_with(|s, e: KeyboardEvent| {
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

#[function_component]
fn Footer() -> Html {
    let active_filter = use_selector(|s: &State| s.filter);

    let filters = {
        let view_filter = |filter: Filter| {
            let cls = if *active_filter == filter {
                "selected"
            } else {
                "not-selected"
            };
            let onclick = Dispatch::<State>::new().reduce_mut_callback(move |s| s.filter = filter);
            html! {
                <li>
                    <a class={cls} href={filter.as_href()} {onclick}>
                        { filter }
                    </a>
                </li>
            }
        };

        let items = [Filter::All, Filter::Active, Filter::Completed]
            .iter()
            .copied()
            .map(view_filter)
            .collect::<Html>();

        html! {
            <ul class="filters">
                { items }
            </ul>
        }
    };

    html! {
        <>
        <TodoCount />
        { filters }
        <BtnClearCompleted />
        </>
    }
}

#[function_component]
fn BtnClearCompleted() -> Html {
    let total_completed = use_selector(|state: &State| state.total_completed());
    let onclick = Dispatch::<State>::new().reduce_mut_callback(|s| s.clear_completed());

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
    let onclick = Dispatch::new().reduce_mut_callback(|s: &mut State| {
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

#[function_component]
fn ListEntries() -> Html {
    let focus_ref = use_node_ref();
    let (state, dispatch) = use_store::<State>();
    let view_entry = |(idx, entry): (usize, &Entry)| {
        let mut class = Classes::from("todo");
        if entry.editing {
            class.push(" editing");
        }
        if entry.completed {
            class.push(" completed");
        }

        let toggle = {
            let onclick = dispatch.reduce_mut_callback(move |s| s.toggle(idx));
            html! {
                <input
                    type="checkbox"
                    class="toggle"
                    checked={entry.completed}
                    {onclick}
                />
            }
        };
        let view = {
            let ondblclick = dispatch.reduce_mut_callback(move |s| {
                s.edit_value = s.entries[idx].description.clone();
                s.clear_all_edit();
                s.toggle_edit(idx);
            });
            let onclick = dispatch.reduce_mut_callback(move |s| s.remove(idx));
            html! {
                <>
                <label {ondblclick}>{ &entry.description }</label>
                <button class="destroy" {onclick} />
                </>
            }
        };
        let edit = {
            let edit = move |input: HtmlInputElement, state: &mut State| {
                let value = input.value();
                input.set_value("");

                state.complete_edit(idx, value.trim().to_string());
                state.edit_value = "".to_string();
            };
            let onblur = dispatch.reduce_mut_callback_with(move |s, e: FocusEvent| {
                edit(e.target_unchecked_into(), s)
            });
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
        };

        html! {
            <li {class}>
                <div class="view">
                    { toggle }
                    { view }
                </div>
                { edit }
            </li>
        }
    };

    let entries = state
        .entries
        .iter()
        .filter(|e| state.filter.fits(e))
        .enumerate()
        .map(view_entry)
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
