use std::rc::Rc;

use yew::{Component, ComponentLink, Html, Properties, ShouldRender};

use crate::{
    component::wrapper::WithDispatcher,
    dispatcher::{Dispatcher, DispatcherProp},
    store::Store,
};

pub type Render<STORE> = Rc<dyn Fn(&Dispatcher<STORE>) -> Html>;
pub type Rendered<STORE> = Rc<dyn Fn(&Dispatcher<STORE>, bool)>;
pub type Change<STORE> = Rc<dyn Fn(&Dispatcher<STORE>, &Dispatcher<STORE>) -> bool>;

#[derive(Properties, Clone)]
pub struct Props<STORE>
where
    STORE: Store + Clone + Default,
{
    #[prop_or_default]
    dispatcher: Dispatcher<STORE>,
    pub view: Render<STORE>,
    #[prop_or_default]
    pub rendered: Option<Rendered<STORE>>,
    #[prop_or_default]
    pub change: Option<Change<STORE>>,
}

impl<STORE> DispatcherProp for Props<STORE>
where
    STORE: Store + Clone + Default,
{
    type Store = STORE;

    fn dispatcher(&mut self) -> &mut Dispatcher<Self::Store> {
        &mut self.dispatcher
    }
}

pub enum Msg {}

pub struct Model<STORE>
where
    STORE: Store + Clone + Default,
{
    props: Props<STORE>,
}

impl<STORE> Component for Model<STORE>
where
    STORE: Store + Default + Clone + 'static,
{
    type Message = Msg;
    type Properties = Props<STORE>;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self { props }
    }

    fn rendered(&mut self, first_render: bool) {
        if let Some(ref f) = self.props.rendered {
            f(&self.props.dispatcher, first_render)
        }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        true
    }

    fn view(&self) -> Html {
        (self.props.view)(&self.props.dispatcher)
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        // Check if other property functions have changed
        let is_eq = Rc::ptr_eq(&self.props.view, &props.view)
            && ptr_eq(&self.props.rendered, &props.rendered)
            && ptr_eq(&self.props.change, &props.change);
        // Update functions if they changed.
        if !is_eq {
            self.props.view = props.view;
            self.props.rendered = props.rendered;
            self.props.change = props.change;
        }
        // Check if state should be updated.
        let should_change = {
            if let Some(ref f) = self.props.change {
                f(&self.props.dispatcher, &props.dispatcher)
            } else {
                // Should change by default.
                true
            }
        };
        // Update state if desired.
        if should_change {
            self.props.dispatcher = props.dispatcher;
        }

        !is_eq || should_change
    }
}

fn ptr_eq<T: ?Sized>(a: &Option<Rc<T>>, b: &Option<Rc<T>>) -> bool {
    a.as_ref()
        .zip(b.as_ref())
        .map(|(a, b)| Rc::ptr_eq(a, b))
        .unwrap_or_default()
}

pub type StateView<H> = WithDispatcher<Model<H>>;

/// Wraps `f` in `Rc`. Helps with resolving type needed for view property.
pub fn view<F, STORE>(f: F) -> Render<STORE>
where
    STORE: Store,
    F: Fn(&Dispatcher<STORE>) -> Html + 'static,
{
    Rc::new(f)
}

/// Wraps `f` in `Rc`. Helps with resolving type needed for rendered property.
pub fn rendered<F, STORE>(f: F) -> Rendered<STORE>
where
    STORE: Store,
    F: Fn(&Dispatcher<STORE>, bool) + 'static,
{
    Rc::new(f)
}

/// Wraps `f` in `Rc`. Helps with resolving type needed for rendered property.
pub fn change<F, STORE>(f: F) -> Change<STORE>
where
    STORE: Store,
    F: Fn(&Dispatcher<STORE>, &Dispatcher<STORE>) -> bool + 'static,
{
    Rc::new(f)
}
