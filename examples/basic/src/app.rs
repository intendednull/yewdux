use yew::{html, Component, ComponentLink, Html, ShouldRender};

use crate::{display::Display, input::Input};

use yewdux::{BasicStore, Dispatch};

pub type AppDispatch = Dispatch<BasicStore<AppState>>;

#[derive(Clone, PartialEq, Default)]
pub struct AppState {
    pub name: String,
}

pub struct App;

impl Component for App {
    type Message = ();
    type Properties = ();

    fn create(_props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <>
                <Display />
                // Components can share state from anywhere!
                <div>
                    <Input />
                </div>
            </>
        }
    }
}
