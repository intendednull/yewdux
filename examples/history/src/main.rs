use yew::prelude::*;
use yewdux::{prelude::*, Context};
use yewdux_utils::*;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct State {
    count: u32,
}

impl Store for State {
    fn new(cx: &Context) -> Self {
        init_listener(HistoryListener::<State>::default, cx);
        Self::default()
    }

    fn should_notify(&self, old: &Self) -> bool {
        self != old
    }
}

#[function_component]
fn App() -> Html {
    let (state, dispatch) = use_store::<State>();
    let on_increment_click = dispatch.reduce_mut_callback(|state| state.count += 1);
    let on_decrement_click = dispatch.reduce_mut_callback(|state| state.count -= 1);

    html! {
        <>
        <p>{ state.count }</p>
        <button onclick={on_increment_click}>{"+1"}</button>
        <button onclick={on_decrement_click}>{"-1"}</button>

        <br/>
        <br/>
        <Controls />
        </>
    }
}

#[function_component]
fn Controls() -> Html {
    let (state, dispatch) = use_store::<HistoryStore<State>>();

    let on_undo_click = dispatch.apply_callback(|_| HistoryMessage::Undo);
    let on_redo_click = dispatch.apply_callback(|_| HistoryMessage::Redo);
    let on_clear_click = dispatch.apply_callback(|_| HistoryMessage::Clear);

    let undo_disabled = !state.can_apply(&HistoryMessage::Undo);
    let redo_disabled = !state.can_apply(&HistoryMessage::Redo);
    let clear_disabled = !state.can_apply(&HistoryMessage::Clear);

    let rows: Html = state
        .states()
        .iter()
        .enumerate()
        .map(|(i, x)| {
            let matches = i == state.index();
            let match_text = if matches { "<<<" } else { "" };
            let text = format!("{x:?}");

            let onclick = dispatch.apply_callback(move |_| HistoryMessage::JumpTo(i));

            html!(<tr><td><button {onclick}>{text}</button></td> <td>{match_text}</td> </tr>)
        })
        .collect();

    html!(
        <div>
            <button onclick={on_undo_click} disabled={undo_disabled}>{"Undo"}</button>
            <button onclick={on_redo_click} disabled={redo_disabled}>{"Redo"}</button>
            <button onclick={on_clear_click} disabled={clear_disabled}>{"Clear History"}</button>

            <table>
            {rows}
            </table>
        </div>
    )
}

fn main() {
    yew::Renderer::<App>::new().render();
}
