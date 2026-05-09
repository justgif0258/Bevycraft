use std::marker::PhantomData;

use parking_lot::{RwLockReadGuard, RwLockWriteGuard};

use crate::prelude::*;

pub trait RegistrarOps<T: Registrable> {
    fn read_from_registry<'a>() -> RwLockReadGuard<'a, impl Registry<T>>;

    fn write_to_registry<'a>() -> RwLockWriteGuard<'a, impl Registry<T>>;
}

pub struct Registrar<T: Registrable>(PhantomData<T>);
