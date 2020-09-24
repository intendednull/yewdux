use std::rc::Rc;

use yew::{Component, ComponentLink, Html, Properties, ShouldRender};

use crate::handle::Handle;
use crate::{SharedState, SharedStateComponent};

pub type Render<H> = Rc<dyn Fn(&H) -> Html>;
pub type Rendered<H> = Rc<dyn Fn(&H, bool)>;

#[derive(Properties, Clone)]
pub struct Props<H>
where
    H: Handle + Clone + Default,
{
    #[prop_or_default]
    handle: H,
    pub view: Render<H>,
    #[prop_or_default]
    pub rendered: Option<Rendered<H>>,
}

impl<H> SharedState for Props<H>
where
    H: Handle + Clone + Default,
{
    type Handle = H;

    fn handle(&mut self) -> &mut Self::Handle {
        &mut self.handle
    }
}

pub enum Msg {}

pub struct Model<H>
where
    H: Handle + Clone + Default,
{
    props: Props<H>,
}

impl<H> Component for Model<H>
where
    H: Handle + Default + Clone + 'static,
{
    type Message = Msg;
    type Properties = Props<H>;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self { props }
    }

    fn rendered(&mut self, first_render: bool) {
        if let Some(ref f) = self.props.rendered {
            f(&self.props.handle, first_render)
        }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        true
    }

    fn view(&self) -> Html {
        (self.props.view)(&self.props.handle)
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }
}

pub type StateView<H, SCOPE = H> = SharedStateComponent<Model<H>, SCOPE>;

/// Wraps `f` in `Rc`. Helps with resolving type needed for view property.
pub fn view<F, H>(f: F) -> Render<H>
where
    F: Fn(&H) -> Html + 'static,
{
    Rc::new(f)
}

/// Wraps `f` in `Rc`. Helps with resolving type needed for rendered property.
pub fn rendered<F, H>(f: F) -> Rendered<H>
where
    F: Fn(&H, bool) + 'static,
{
    Rc::new(f)
}
