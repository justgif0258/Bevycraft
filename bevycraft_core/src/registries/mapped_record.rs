use bevy::ecs::resource::Resource;
use boomphf::Mphf;

use crate::prelude::{AssetLocation, Commit, Record, Recordable};

#[derive(Resource, Debug)]
pub struct MappedRecord<T: Recordable> {
    m_hasher: Mphf<AssetLocation>,
    entries: Box<[(AssetLocation, T)]>,
}

impl<T: Recordable> MappedRecord<T> {
    const BASE: f64 = 3.3f64;

    fn gen_boxed_entries<C>(phf: &Mphf<AssetLocation>, commit: C) -> Box<[(AssetLocation, T)]>
    where
        C: Commit<Value = T>,
    {
        let mut boxed = Box::<[(AssetLocation, T)]>::new_uninit_slice(commit.len());

        commit.into_iter().for_each(|(key, value)| {
            let idx = phf.hash(&key) as usize;

            boxed[idx].write((key, value));
        });

        unsafe { boxed.assume_init() }
    }

    fn gen_phf(keys: Vec<AssetLocation>) -> Mphf<AssetLocation> {
        Mphf::new(Self::BASE, keys.as_slice())
    }
}

impl<T: Recordable> Record for MappedRecord<T> {
    type Value = T;
    
    type Index = usize;

    fn finish<C>(commit: C) -> Self
    where
        C: Commit<Value = Self::Value>,
    {
        let keys = commit.iter_keys().cloned().collect();

        let m_hasher = Self::gen_phf(keys);

        let entries = Self::gen_boxed_entries(&m_hasher, commit);

        Self { m_hasher, entries }
    }

    #[inline]
    fn get_by_key(&self, key: &AssetLocation) -> Option<&Self::Value> {
        let idx = self.m_hasher.try_hash(key)?;

        self.entries.get(idx as usize).and_then(|(k, v)| {
            if k != key {
                return None;
            }

            Some(v)
        })
    }

    #[inline]
    fn get_by_idx(&self, index: Self::Index) -> Option<&Self::Value> {
        self.entries.get(index).map(|(_, v)| v)
    }

    #[inline]
    fn key_to_idx(&self, key: &AssetLocation) -> Option<Self::Index> {
        self.m_hasher.try_hash(key).and_then(|idx| {
            if &self.entries[idx as usize].0 != key {
                return None;
            }

            Some(idx as usize)
        })
    }

    #[inline]
    fn idx_to_key(&self, index: usize) -> Option<&AssetLocation> {
        self.entries.get(index).map(|(k, _)| k)
    }

    #[inline]
    fn iter(&self) -> impl Iterator<Item = &(AssetLocation, Self::Value)> {
        self.entries.iter()
    }

    #[inline]
    fn iter_keys(&self) -> impl Iterator<Item = &AssetLocation> {
        self.entries.iter().map(|(k, _)| k)
    }

    #[inline]
    fn len(&self) -> usize {
        self.entries.len()
    }
}
