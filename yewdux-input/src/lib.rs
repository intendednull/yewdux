use gloo::file::{File, FileReadError};
use web_sys::HtmlInputElement;
use yew::{
    events::{Event, InputEvent},
    prelude::{Callback, TargetCast},
    FocusEvent,
};

use yewdux::{dispatch::Dispatcher, store::Store};

pub trait InputDispatcher: Dispatcher {
    /// Callback submitting a form. Disables default event behavior for forms.
    fn submit(
        &self,
        f: impl Fn(&mut <Self::Store as Store>::Model) + 'static,
    ) -> Callback<FocusEvent> {
        self.reduce_callback_with(move |s, e: FocusEvent| {
            e.prevent_default();
            f(s)
        })
    }

    /// Callback that sets state, ignoring callback event.
    fn set<E: 'static>(
        &self,
        f: impl FnOnce(&mut <Self::Store as Store>::Model) + 'static,
    ) -> Callback<E> {
        self.reduce_callback_once(f)
    }

    /// Callback that sets state from callback event
    fn set_with<E: 'static>(
        &self,
        f: impl FnOnce(&mut <Self::Store as Store>::Model, E) + 'static,
    ) -> Callback<E> {
        self.reduce_callback_once_with(f)
    }

    /// Callback for setting state from `InputData`.
    fn on_input(
        &self,
        f: impl Fn(&mut <Self::Store as Store>::Model, String) + 'static,
    ) -> Callback<InputEvent> {
        self.reduce_callback_with(f).reform(|e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            input.value()
        })
    }

    /// Callback for setting state from `InputData`.
    fn on_input_once(
        &self,
        f: impl FnOnce(&mut <Self::Store as Store>::Model, String) + 'static,
    ) -> Callback<InputEvent> {
        self.reduce_callback_once_with(f).reform(|e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            input.value()
        })
    }

    /// Callback for setting state from `InputData`.
    fn on_change(
        &self,
        f: impl Fn(&mut <Self::Store as Store>::Model, String) + 'static,
    ) -> Callback<Event> {
        self.reduce_callback_with(f).reform(|e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            input.value()
        })
    }

    /// Callback for setting state from `InputData`.
    fn on_change_once(
        &self,
        f: impl FnOnce(&mut <Self::Store as Store>::Model, String) + 'static,
    ) -> Callback<Event> {
        self.reduce_callback_once_with(f).reform(|e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            input.value()
        })
    }

    /// Callback for setting files
    fn file(
        &self,
        f: impl Fn(&mut <Self::Store as Store>::Model, Result<Vec<u8>, FileReadError>) + Copy + 'static,
    ) -> Callback<Event> {
        let set_file = self.set_with(f);
        Callback::from(move |e: Event| {
            let input: HtmlInputElement = e.target_unchecked_into();
            if let Some(files) = input.files() {
                for file in js_sys::try_iter(&files)
                    .unwrap()
                    .unwrap()
                    .into_iter()
                    .map(|v| web_sys::File::from(v.unwrap()))
                    .map(File::from)
                {
                    let cb = set_file.clone();
                    gloo::file::callbacks::read_as_bytes(&file, move |result| {
                        cb.emit(result);
                    });
                }
            }
        })
    }
}

impl<T: Dispatcher> InputDispatcher for T {}
