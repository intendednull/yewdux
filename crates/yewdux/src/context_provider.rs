use yew::prelude::*;

use crate::context;

#[derive(PartialEq, Clone, Properties)]
pub struct Props {
    pub children: Children,
}

#[function_component]
pub fn YewduxRoot(Props { children }: &Props) -> Html {
    let ctx = use_state(context::Context::new);
    html! {
        <ContextProvider<context::Context> context={(*ctx).clone()}>
            { children.clone() }
        </ContextProvider<context::Context>>
    }
}
