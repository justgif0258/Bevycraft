use parking_lot::{RwLockReadGuard, RwLockWriteGuard};

use crate::prelude::*;

pub trait Registrar<'a> {
    type Item: Registrable;

    fn read_from_registry() -> RwLockReadGuard<'a, impl Registry<Item = Self::Item>>;

    fn write_to_registry() -> RwLockWriteGuard<'a, impl Registry<Item = Self::Item>>;
}
