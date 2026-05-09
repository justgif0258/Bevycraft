use std::{marker::PhantomData, ops::Deref, sync::OnceLock};

use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct Holder<'a, T> {
    id: OnceLock<usize>,
    key: &'a str,
    factory: fn() -> T,
    _marker: PhantomData<&'a T>,
}

impl<'a, T> Deref for Holder<'a, T> {
    type Target = usize;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.id
            .get()
            .expect("Tried dereferencing unregistered holder")
    }
}

impl<'a, T> Holder<'a, T>
where
    T: Registrar<'a>,
{
    pub fn registrar(&self, registry: &mut <T as Registrar<'a>>::Registry) {
        let location = AssetLocation::parse(self.key);

        registry
            .register(location.clone(), (self.factory)())
            .expect("Registration failed");

        let id = registry.key_to_idx(&location).unwrap();
        self.id.set(id).unwrap();
    }
}

impl<'a, T> Holder<'a, T> {
    pub const fn new(key: &'a str, factory: fn() -> T) -> Self {
        Self {
            id: OnceLock::new(),
            key,
            factory,
            _marker: PhantomData,
        }
    }
}
