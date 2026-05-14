use {
    crate::prelude::*,
    parking_lot::{RwLockReadGuard, RwLockWriteGuard},
    std::marker::PhantomData,
};

pub trait RegistrarOps<T: Registrable> {
    fn read_from_registry<'a>() -> RwLockReadGuard<'a, impl Registry<T>>;

    fn write_to_registry<'a>() -> RwLockWriteGuard<'a, impl Registry<T>>;
}

pub struct Registrar<T: Registrable>(PhantomData<T>);
