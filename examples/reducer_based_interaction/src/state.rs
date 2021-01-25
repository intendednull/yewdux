use yew_state::handler::{StateHandler, HandlerLink};
use yew::agent::HandlerId;
use yew_state::reducer_handler::Reducer;


#[derive(Clone)]
pub struct GamesState{
    pub games:Vec<String>,
}
#[derive(Clone)]
pub enum GameStateAction{
    AddGame(String)
}
impl Reducer for GamesState{
    type Action = GameStateAction;

    fn init() -> Self {
        GamesState{
            games:vec![]
        }
    }

    fn reduce(&mut self, action: Self::Action)->bool{
        match action {
            GameStateAction::AddGame(game)=>{
                self.games.push(game);
                true
            }
        }
    }
}
