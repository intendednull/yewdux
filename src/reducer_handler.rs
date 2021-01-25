use std::rc::Rc;
use yew::agent::HandlerId;
use crate::service::{ServiceBridge, ServiceOutput, ServiceResponse};
use yew::{ComponentLink, Component, Callback};
use crate::handler::{StateHandler, HandlerLink};

pub struct ReducerHandler<T> where T: Reducer + Clone {
    state:Rc<T>
}
pub trait Reducer {
    type Action:Clone;
    fn reduce(&mut self,action:Self::Action)->bool;
    fn init()->Self;
}
impl<T> StateHandler for ReducerHandler<T> where T: Reducer + Clone {
    type Model = T;
    type Message = ();
    type Input = T::Action;
    type Output = ();

    fn new(_link: HandlerLink<Self>) -> Self {
        Self{
            state:Rc::new(T::init())
        }
    }

    fn state(&mut self) -> &mut Rc<Self::Model> {
        &mut self.state
    }

    fn handle_input(&mut self, msg: Self::Input, _who: HandlerId) -> bool {
        let mut state = Rc::make_mut(&mut self.state);
        state.reduce(msg)
    }
}
pub struct ReducerBridge<T> where T: Reducer + Clone + 'static{
    bridge:ServiceBridge<ReducerHandler<T>>
}
impl<T> ReducerBridge<T> where T: Reducer + Clone +'static {
    pub fn dispatch(&mut self,action:T::Action){
        self.bridge.send_handler(action);
    }
    pub fn dispatcher(comp_callback:Callback<Rc<T>>)->Self{
        let func = Rc::new(move |so|{
            match so {
                ServiceOutput::Handler(msg) => match msg {
                    _ => unreachable!(),
                },
                // Messages sent by StateService.
                ServiceOutput::Service(msg) => match msg {
                    // This is received every time state changes.
                    ServiceResponse::State(state) => {comp_callback.emit(state);},
                    // This is received once, when first connecting. Handy if you want access to all the
                    // handler link methods.
                    ServiceResponse::Link(link) => {},
                },
            }
        });
        let callback= Callback::Callback(func.clone());
        Self{
            bridge:ServiceBridge::new(callback)
        }
    }
}