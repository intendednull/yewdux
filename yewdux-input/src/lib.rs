use yew::{
    prelude::{Callback, InputData},
    ChangeData, FocusEvent,
};
use yew_services::reader::{File, FileData, ReaderService};
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
    fn input(
        &self,
        f: impl Fn(&mut <Self::Store as Store>::Model, String) + 'static,
    ) -> Callback<InputData> {
        self.reduce_callback_with(f)
            .reform(|data: InputData| data.value)
    }

    /// Callback for setting state from `InputData`.
    fn input_once(
        &self,
        f: impl FnOnce(&mut <Self::Store as Store>::Model, String) + 'static,
    ) -> Callback<InputData> {
        self.reduce_callback_once_with(f)
            .reform(|data: InputData| data.value)
    }

    /// Callback for setting state from `InputData`.
    fn text(
        &self,
        f: impl Fn(&mut <Self::Store as Store>::Model, String) + 'static,
    ) -> Callback<ChangeData> {
        let on_change = self.reduce_callback_with(f);
        Callback::from(move |data: ChangeData| {
            if let ChangeData::Value(val) = data {
                on_change.emit(val);
            }
        })
    }

    /// Callback for setting state from `InputData`.
    fn text_once(
        &self,
        f: impl FnOnce(&mut <Self::Store as Store>::Model, String) + 'static,
    ) -> Callback<ChangeData> {
        let on_change = self.reduce_callback_once_with(f);
        Callback::from(move |data: ChangeData| {
            if let ChangeData::Value(val) = data {
                on_change.emit(val);
            }
        })
    }

    fn select(
        &self,
        f: impl Fn(&mut <Self::Store as Store>::Model, String) + 'static,
    ) -> Callback<ChangeData> {
        let on_change = self.reduce_callback_with(f);
        Callback::from(move |data: ChangeData| {
            if let ChangeData::Select(el) = data {
                on_change.emit(el.value());
            }
        })
    }

    fn select_once(
        &self,
        f: impl FnOnce(&mut <Self::Store as Store>::Model, String) + 'static,
    ) -> Callback<ChangeData> {
        let on_change = self.reduce_callback_once_with(f);
        Callback::from(move |data: ChangeData| {
            if let ChangeData::Select(el) = data {
                on_change.emit(el.value());
            }
        })
    }

    /// Callback for setting files
    fn file(
        &self,
        f: impl Fn(&mut <Self::Store as Store>::Model, FileData) + Copy + 'static,
    ) -> Callback<ChangeData> {
        let set_file = self.set_with(f);
        Callback::from(move |data| {
            if let ChangeData::Files(files) = data {
                for file in js_sys::try_iter(&files)
                    .unwrap()
                    .unwrap()
                    .into_iter()
                    .map(|v| File::from(v.unwrap()))
                {
                    ReaderService::read_file(file, set_file.clone()).ok();
                }
            }
        })
    }
}

impl<T: Dispatcher> InputDispatcher for T {}
