use bevy::platform::collections::hash_map::IntoIter;
use bevy::platform::collections::HashMap;
use bevy::prelude::Resource;
use bevycraft_core::prelude::{AssetLocation, Commit};
use crate::prelude::Block;

#[derive(Resource)]
pub struct BlockCommit {
    entries: HashMap<AssetLocation, Block>,
}

impl BlockCommit {
    #[inline]
    pub const fn new() -> Self {
        Self { entries: HashMap::new() }
    }

    #[inline(always)]
    fn contains(&self, location: &AssetLocation) -> bool {
        self.entries.contains_key(location)
    }
}

impl IntoIterator for BlockCommit {
    type Item = (AssetLocation, Block);
    type IntoIter = IntoIter<AssetLocation, Block>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}

impl Commit for BlockCommit {
    type Value = Block;

    #[inline]
    fn push(&mut self, key: AssetLocation, recordable: Self::Value) {
        assert!(!self.contains(&key), "Tried pushing duplicated key '{}'", key);

        self.entries.insert(key, recordable);
    }

    #[inline]
    fn pop(&mut self, key: &AssetLocation) -> Option<(AssetLocation, Self::Value)> {
        assert!(self.contains(key), "Tried removing non-existing key '{}'", key);

        self.entries.remove_entry(key)
    }

    #[inline]
    fn merge<C>(&mut self, other: C)
    where
        C: Commit<Value=Self::Value>
    {
        other.into_iter()
            .for_each(|(key, block)| {
                assert!(!self.contains(&key), "Tried pushing duplicated key '{}'", key);

                self.entries.insert(key, block);
            })
    }

    #[inline]
    fn iter_keys(&self) -> impl Iterator<Item = &AssetLocation> {
        self.entries.keys()
    }

    #[inline]
    fn len(&self) -> usize {
        self.entries.len()
    }
}