use std::{rc::Rc, str::FromStr};

use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, HtmlTextAreaElement};
use yew::prelude::*;
use yewdux::prelude::*;
use serde::{Deserialize, Serialize};

pub enum InputElement {
    Input(HtmlInputElement),
    TextArea(HtmlTextAreaElement),
}

pub trait FromInputElement: Sized {
    fn from_input_element(el: InputElement) -> Option<Self>;
}

impl<T> FromInputElement for T
where
    T: FromStr,
{
    fn from_input_element(el: InputElement) -> Option<Self> {
        match el {
            InputElement::Input(el) => el.value().parse().ok(),
            InputElement::TextArea(el) => el.value().parse().ok(),
        }
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Checkbox(bool);

impl Checkbox {
    pub fn checked(&self) -> bool {
        self.0
    }
}

impl FromInputElement for Checkbox {
    fn from_input_element(el: InputElement) -> Option<Self> {
        if let InputElement::Input(el) = el {
            Some(Self(el.checked()))
        } else {
            None
        }
    }
}

pub trait InputDispatch<S: Store> {
    fn input<F, E, R>(&self, f: F) -> Callback<E>
    where
        S: Clone,
        R: FromInputElement,
        F: Fn(Rc<S>, R) -> Rc<S> + 'static,
        E: AsRef<Event> + JsCast + 'static,
    {
        Dispatch::<S>::new().reduce_callback_with(move |s, e| {
            if let Some(value) = input_value(e) {
                f(s, value)
            } else {
                s
            }
        })
    }

    fn input_mut<F, E, R>(&self, f: F) -> Callback<E>
    where
        S: Clone,
        R: FromInputElement,
        F: Fn(&mut S, R) + 'static,
        E: AsRef<Event> + JsCast + 'static,
    {
        Dispatch::<S>::new().reduce_mut_callback_with(move |s, e| {
            if let Some(value) = input_value(e) {
                f(s, value);
            }
        })
    }
}

impl<S: Store> InputDispatch<S> for Dispatch<S> {}

/// Get any parsable value out of an input event.
pub fn input_value<E, R>(event: E) -> Option<R>
where
    R: FromInputElement,
    E: AsRef<Event> + JsCast,
{
    event
        .target_dyn_into::<HtmlInputElement>()
        .and_then(|el| R::from_input_element(InputElement::Input(el)))
        .or_else(|| {
            event
                .target_dyn_into::<HtmlTextAreaElement>()
                .and_then(|el| R::from_input_element(InputElement::TextArea(el)))
        })
}
