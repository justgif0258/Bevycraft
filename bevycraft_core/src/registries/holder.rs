use std::{marker::PhantomData, mem::transmute, ops::Deref, sync::OnceLock};

use parking_lot::{MappedRwLockReadGuard, RwLockReadGuard};

use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct Holder<'a, T> {
    id: OnceLock<usize>,
    key: &'a str,
    factory: fn() -> T,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Registrable> Deref for Holder<'a, T> {
    type Target = usize;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.id
            .get()
            .expect("Cannot dereference unregistered holder")
    }
}

impl<'a, T: Registrable> Holder<'a, T> {
    pub const fn new(key: &'a str, factory: fn() -> T) -> Self {
        Self {
            id: OnceLock::new(),
            key,
            factory,
            _marker: PhantomData,
        }
    }

    pub fn registrar(&self, registry: &mut impl Registry<T>) {
        let location = AssetLocation::parse(self.key);

        registry
            .register(location.clone(), (self.factory)())
            .expect("Registration failed");

        let id = registry.key_to_idx(&location).unwrap();
        self.id.set(id).unwrap();
    }

    #[inline]
    pub fn get(&self) -> MappedRwLockReadGuard<'a, T>
    where
        Registrar<T>: RegistrarOps<T>,
    {
        let guard = RwLockReadGuard::map(Registrar::<T>::read_from_registry(), |registry| {
            registry.get_by_idx(**self).unwrap()
        });

        unsafe { transmute(guard) }
    }
}
