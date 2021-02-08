mod components;
mod state;
extern crate yew_state;
use yew::prelude::*;
use components::Games;
pub struct App {
    link: ComponentLink<Self>,
}


impl Component for App {
    type Message = ();
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        App {
            link,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        true
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <Games/>
            </div>
        }
    }
}
fn main() {
    yew::start_app::<App>();
}