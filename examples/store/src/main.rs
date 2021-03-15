use std::rc::Rc;

use yew::prelude::*;
use yewdux::prelude::*;

#[derive(Clone)]
struct State {
    count: u32,
}

enum CounterInput {
    /// Increment count by one.
    Increment,
}

enum CounterOutput {
    /// Output the current count but doubled.
    Doubled(u32),
}

struct CounterStore {
    state: Rc<State>,
    link: StoreLink<Self>,
}

impl Store for CounterStore {
    type Model = State;
    type Message = ();
    type Input = CounterInput;
    type Output = CounterOutput;

    fn new(link: StoreLink<Self>) -> Self {
        Self {
            link,
            state: Rc::new(State { count: 0 }),
        }
    }

    fn state(&mut self) -> &mut Rc<Self::Model> {
        &mut self.state
    }

    fn handle_input(&mut self, msg: Self::Input, who: HandlerId) -> Changed {
        let state = Rc::make_mut(&mut self.state);
        match msg {
            CounterInput::Increment => {
                state.count += 1;
                // Response with current count doubled.
                self.link
                    .respond(who, CounterOutput::Doubled(state.count * 2));
            }
        }

        true
    }
}

enum Msg {
    Output(CounterOutput),
    State(Rc<State>),
}

struct App {
    dispatch: Dispatch<CounterStore>,
    state: Rc<State>,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let dispatch = {
            let on_state = link.callback(Msg::State);
            let on_output = link.callback(Msg::Output);

            Dispatch::bridge(on_state, on_output)
        };
        // Magically increment counter by one for this example.
        dispatch.send(CounterInput::Increment);

        Self {
            dispatch,
            state: Rc::new(State { count: 0 }),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::State(state) => {
                self.state = state;
                true
            }
            Msg::Output(msg) => match msg {
                CounterOutput::Doubled(n) => {
                    println!("Count doubled would be: {}", n);
                    false
                }
            },
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let count = self.state.count;
        let onclick = self.dispatch.callback(|_| CounterInput::Increment);
        html! {
            <>
            <h1>{ count }</h1>
            <button onclick=onclick>{"+1"}</button>
            </>
        }
    }
}

fn main() {
    yew::start_app::<App>();
}
