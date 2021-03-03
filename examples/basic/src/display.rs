use yew::{html, Component, ComponentLink, Html, ShouldRender};
use yewdux::WithDispatch;
use yewtil::NeqAssign;

use crate::app::AppDispatch;

pub struct Model {
    dispatch: AppDispatch,
}

impl Component for Model {
    type Message = ();
    type Properties = AppDispatch;

    fn create(dispatch: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Model { dispatch }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, handle: Self::Properties) -> ShouldRender {
        self.dispatch.neq_assign(handle)
    }

    fn view(&self) -> Html {
        let name = &self.dispatch.state().name;
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

pub type Display = WithDispatch<Model>;
