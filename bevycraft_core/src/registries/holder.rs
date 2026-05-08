use std::{
    marker::PhantomData,
    ops::Deref,
    sync::{LazyLock, OnceLock},
};

use crate::prelude::*;

pub struct Holder<'a, T> {
    id: OnceLock<usize>,
    key: LazyLock<AssetLocation>,
    factory: fn() -> T,
    _marker: PhantomData<&'a T>,
}

impl<'a, T> AsRef<T> for Holder<'a, T>
where
    T: Registrar,
{
    #[inline(always)]
    fn as_ref(&self) -> &T {
        let reg = T::read_from_registry();

        let t = reg.get_by_idx(self.id.get().copied().unwrap()).unwrap();

        // SAFETY: It is garanteed that the value is still alive by the lifetime constraints
        unsafe { &*(t as *const T) }
    }
}

impl<'a, T> Deref for Holder<'a, T> {
    type Target = usize;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.id.get().unwrap()
    }
}

impl<'a> Holder<'a, Block> {
    #[inline(always)]
    pub fn get_type(&self) -> BlockType {
        BlockType::new(self.id.get().copied().unwrap() as u32)
    }
}

impl<'a, T> Holder<'a, T>
where
    T: Registrar,
{
    pub fn registrar(&self) {
        let mut reg = T::write_to_registry();

        reg.register(self.key.clone(), (self.factory)())
            .expect("Registration failed");
    }
}

impl<'a, T> Holder<'a, T> {
    pub const fn new(location: fn() -> AssetLocation, factory: fn() -> T) -> Self {
        Self {
            id: OnceLock::new(),
            key: LazyLock::new(location),
            factory,
            _marker: PhantomData,
        }
    }
}
