use yew::prelude::*;
use yew_state::service::{ServiceBridge, ServiceOutput, ServiceResponse};
use std::rc::Rc;
use crate::state::{GamesState, GameStateAction};
use yew_state::reducer_handler::{ReducerHandler, Reducer, ReducerBridge};

pub struct Games {
    game_state:GamesState,
    dispatcher:ReducerBridge<GamesState>,
    link:ComponentLink<Self>
}
#[derive(Clone)]
pub enum GamesMessage{
    AddGame(String),
    RefreshState(GamesState)
}
impl Component for Games {
    type Message = GamesMessage;
    type Properties = ();

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let callback = link.callback(|state:Rc<GamesState>|{Self::Message::RefreshState((*state).clone())});
        Self{
            game_state:GamesState::init(),
            dispatcher:ReducerBridge::dispatcher(callback),
            link
        }
    }

    fn update(&mut self, msg: Self::Message) -> bool {
        match msg {
            Self::Message::RefreshState(state)=>{
                self.game_state = state;
                true
            },
            Self::Message::AddGame(game)=>{
                self.dispatcher.dispatch(GameStateAction::AddGame(game));
                false
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> bool {
        unimplemented!()
    }

    fn view(&self) -> Html {
        let add_game = self.link.callback(|_|{Self::Message::AddGame(format!("New One"))});
        let games:Vec<Html> = self.game_state.games.iter().map(|game|{
            html!{
                <div>{&game}</div>
            }
        }).collect();
        html! {
            <>
            <h1>{"Games"}</h1>
            <button onclick=add_game>{"Add Game"}</button>
            <div>
            {games}
            </div>
            </>
        }
    }
}
