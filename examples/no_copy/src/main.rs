use std::cell::RefCell;
use std::rc::Rc;

use yew::{prelude::*, utils::NeqAssign};
use yewdux::prelude::*;

#[derive(Default)]
struct State {
    count: u32,
}

enum CountMsg {
    Increment,
}

struct CounterStore {
    state: Rc<Rc<RefCell<State>>>,
}

impl Store for CounterStore {
    type Model = Rc<RefCell<State>>;
    type Message = ();
    type Input = CountMsg;
    type Output = ();

    fn new(_link: StoreLink<Self>) -> Self {
        Self {
            state: Default::default(),
        }
    }

    fn state(&mut self) -> &mut Rc<Self::Model> {
        &mut self.state
    }

    fn handle_input(&mut self, msg: Self::Input, _who: HandlerId) -> Changed {
        // IMPORTANT: This changes the outer Rc pointer, so subscribers can determine if state was
        // modified. Otherwise DispatchProps won't work.
        let state = Rc::make_mut(&mut self.state);

        match msg {
            CountMsg::Increment => {
                state.borrow_mut().count += 1;
            }
        }

        true
    }
}

struct Model {
    dispatch: DispatchProps<CounterStore>,
}

impl Component for Model {
    type Message = ();
    type Properties = DispatchProps<CounterStore>;

    fn create(dispatch: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self { dispatch }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        false
    }

    fn change(&mut self, handle: Self::Properties) -> ShouldRender {
        self.dispatch.neq_assign(handle)
    }

    fn view(&self) -> Html {
        let count = self.dispatch.state().borrow().count;
        let incr = self.dispatch.callback(|_| CountMsg::Increment);
        let double = self.dispatch.reduce_callback(|s| s.borrow_mut().count *= 2);
        html! {
            <>
            <h1>{ count }</h1>
            <button onclick={incr}>{"+1"}</button>
            <button onclick={double}>{"x2"}</button>
            </>
        }
    }
}

type App = WithDispatch<Model>;

fn main() {
    yew::start_app::<App>();
}
