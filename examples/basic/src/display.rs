use yew::{html, Component, ComponentLink, Html, ShouldRender};
use yew_state::{SharedHandle, SharedStateComponent};
use yewtil::NeqAssign;

use crate::app::AppState;

pub struct Model {
    handle: SharedHandle<AppState>,
}

impl Component for Model {
    type Message = ();
    type Properties = SharedHandle<AppState>;

    fn create(handle: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Model { handle }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, handle: Self::Properties) -> ShouldRender {
        self.handle.neq_assign(handle)
    }

    fn view(&self) -> Html {
        let name = &self.handle.state().name;
        let name = if name.is_empty() {
            "Stranger".to_string()
        } else {
            name.clone()
        };

        html! {
            <p>{ format!("Hello, {}!", name) }</p>
        }
    }
}

pub type Display = SharedStateComponent<Model>;
