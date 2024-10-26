use std::{marker::PhantomData, rc::Rc};
use yewdux::{prelude::*, Context};

#[derive(Default)]
pub struct HistoryListener<T: Store + PartialEq>(PhantomData<T>);

struct HistoryChangeMessage<T: Store + PartialEq>(Rc<T>);

impl<T: Store + PartialEq> Reducer<HistoryStore<T>> for HistoryChangeMessage<T> {
    fn apply(self, mut state: Rc<HistoryStore<T>>) -> Rc<HistoryStore<T>> {
        if state.matches_current(&self.0) {
            return state;
        }

        let mut_state = Rc::make_mut(&mut state);
        mut_state.index += 1;
        mut_state.vector.truncate(mut_state.index);
        mut_state.vector.push(self.0);

        state
    }
}

impl<T: Store + PartialEq> Listener for HistoryListener<T> {
    type Store = T;

    fn on_change(&self, cx: &Context, state: Rc<Self::Store>) {
        Dispatch::<HistoryStore<T>>::new(cx).apply(HistoryChangeMessage::<T>(state))
    }
}

#[derive(Debug, PartialEq)]
pub struct HistoryStore<T: Store + PartialEq> {
    vector: Vec<Rc<T>>,
    index: usize,
    dispatch: Dispatch<T>,
}

impl<T: Store + PartialEq> Clone for HistoryStore<T> {
    fn clone(&self) -> Self {
        Self {
            vector: self.vector.clone(),
            index: self.index,
            dispatch: self.dispatch.clone(),
        }
    }
}

impl<T: Store + PartialEq> HistoryStore<T> {
    pub fn can_apply(&self, message: &HistoryMessage) -> bool {
        match message {
            HistoryMessage::Undo => self.index > 0,
            HistoryMessage::Redo => self.index + 1 < self.vector.len(),
            HistoryMessage::Clear => self.vector.len() > 1,
            HistoryMessage::JumpTo(index) => index != &self.index && index < &self.vector.len(),
        }
    }

    fn matches_current(&self, state: &Rc<T>) -> bool {
        let c = self.current();
        Rc::ptr_eq(c, state)
    }

    fn current(&self) -> &Rc<T> {
        &self.vector[self.index]
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn states(&self) -> &[Rc<T>] {
        self.vector.as_slice()
    }
}

impl<T: Store + PartialEq> Store for HistoryStore<T> {
    fn new(cx: &Context) -> Self {
        let dispatch = Dispatch::<T>::new(cx);
        let s1 = dispatch.get();
        Self {
            vector: vec![s1],
            index: 0,
            dispatch,
        }
    }

    fn should_notify(&self, other: &Self) -> bool {
        self != other
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryMessage {
    Undo,
    Redo,
    Clear,
    JumpTo(usize),
}

impl<T: Store + PartialEq + Clone> Reducer<HistoryStore<T>> for HistoryMessage {
    fn apply(self, mut state: Rc<HistoryStore<T>>) -> Rc<HistoryStore<T>> {
        let mut_state = Rc::make_mut(&mut state);

        let state_changed = match self {
            HistoryMessage::Undo => {
                if let Some(new_index) = mut_state.index.checked_sub(1) {
                    mut_state.index = new_index;
                    true
                } else {
                    false
                }
            }
            HistoryMessage::Redo => {
                let new_index = mut_state.index + 1;
                if new_index < mut_state.vector.len() {
                    mut_state.index = new_index;
                    true
                } else {
                    false
                }
            }
            HistoryMessage::Clear => {
                let current = mut_state.vector[mut_state.index].clone();
                mut_state.vector.clear();
                mut_state.vector.push(current);
                mut_state.index = 0;
                false
            }
            HistoryMessage::JumpTo(index) => {
                if index < mut_state.vector.len() {
                    mut_state.index = index;

                    true
                } else {
                    false
                }
            }
        };

        if state_changed {
            mut_state.dispatch.reduce(|_| mut_state.current().clone());
        }

        state
    }
}
