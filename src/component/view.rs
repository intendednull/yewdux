use std::rc::Rc;

use yew::{Component, ComponentLink, Html, Properties, ShouldRender};

use crate::{
    component::wrapper::WithDispatch,
    dispatch::{Dispatch, DispatchProp},
    store::Store,
};

pub type Render<STORE> = Rc<dyn Fn(&Dispatch<STORE>) -> Html>;
pub type Rendered<STORE> = Rc<dyn Fn(&Dispatch<STORE>, bool)>;
pub type Change<STORE> = Rc<dyn Fn(&Dispatch<STORE>, &Dispatch<STORE>) -> bool>;

#[derive(Properties, Clone)]
pub struct Props<STORE>
where
    STORE: Store + Clone + Default,
{
    #[prop_or_default]
    dispatch: Dispatch<STORE>,
    pub view: Render<STORE>,
    #[prop_or_default]
    pub rendered: Option<Rendered<STORE>>,
    #[prop_or_default]
    pub change: Option<Change<STORE>>,
}

impl<STORE> DispatchProp for Props<STORE>
where
    STORE: Store + Clone + Default,
{
    type Store = STORE;

    fn dispatch(&mut self) -> &mut Dispatch<Self::Store> {
        &mut self.dispatch
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
            f(&self.props.dispatch, first_render)
        }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        true
    }

    fn view(&self) -> Html {
        (self.props.view)(&self.props.dispatch)
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
                f(&self.props.dispatch, &props.dispatch)
            } else {
                // Should change by default.
                true
            }
        };
        // Update state if desired.
        if should_change {
            self.props.dispatch = props.dispatch;
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

pub type StateView<H> = WithDispatch<Model<H>>;

/// Wraps `f` in `Rc`. Helps with resolving type needed for view property.
pub fn view<F, STORE>(f: F) -> Render<STORE>
where
    STORE: Store,
    F: Fn(&Dispatch<STORE>) -> Html + 'static,
{
    Rc::new(f)
}

/// Wraps `f` in `Rc`. Helps with resolving type needed for rendered property.
pub fn rendered<F, STORE>(f: F) -> Rendered<STORE>
where
    STORE: Store,
    F: Fn(&Dispatch<STORE>, bool) + 'static,
{
    Rc::new(f)
}

/// Wraps `f` in `Rc`. Helps with resolving type needed for rendered property.
pub fn change<F, STORE>(f: F) -> Change<STORE>
where
    STORE: Store,
    F: Fn(&Dispatch<STORE>, &Dispatch<STORE>) -> bool + 'static,
{
    Rc::new(f)
}
