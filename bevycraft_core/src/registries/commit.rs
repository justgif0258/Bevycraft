use bevy::ecs::resource::Resource;

use crate::prelude::*;

pub trait Commit: Resource + IntoIterator<Item = (AssetLocation, Self::Value)> {
    type Value: Recordable;

    fn push(&mut self, key: AssetLocation, recordable: Self::Value);

    fn pop(&mut self, key: &AssetLocation) -> Option<(AssetLocation, Self::Value)>;

    fn merge<C>(&mut self, other: C)
    where
        C: Commit<Value = Self::Value>;

    fn keys(&self) -> Vec<&AssetLocation>;

    fn cloned_keys(&self) -> Vec<AssetLocation>;

    fn len(&self) -> usize;
}
