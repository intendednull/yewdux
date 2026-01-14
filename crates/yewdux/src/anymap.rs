use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

#[derive(Default)]
pub(crate) struct AnyMap {
    map: HashMap<TypeId, Box<dyn Any>>,
}

impl AnyMap {
    pub(crate) fn entry<T: 'static>(&mut self) -> Entry<'_, T> {
        Entry {
            map: &mut self.map,
            _marker: std::marker::PhantomData,
        }
    }
}

pub(crate) struct Entry<'a, T: 'static> {
    map: &'a mut HashMap<TypeId, Box<dyn Any>>,
    _marker: std::marker::PhantomData<T>,
}

impl<'a, T: 'static> Entry<'a, T> {
    pub(crate) fn or_insert_with<F>(self, default: F) -> &'a mut T
    where
        F: FnOnce() -> T,
    {
        let type_id = TypeId::of::<T>();
        let value = self
            .map
            .entry(type_id)
            .or_insert_with(|| Box::new(default()));
        value.downcast_mut().expect("type id mismatch")
    }
}
