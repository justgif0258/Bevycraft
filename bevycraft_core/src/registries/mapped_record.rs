use bevy::ecs::resource::Resource;
use boomphf::Mphf;

use crate::prelude::{AssetLocation, Commit, Entry, Record, Recordable};

#[derive(Resource, Debug)]
pub struct MappedRecord<T: Recordable> {
    m_hasher: Mphf<AssetLocation>,
    entries: Box<[Entry<T>]>,
}

impl<T: Recordable> MappedRecord<T> {
    pub const BASE: f64 = 3.3f64;

    pub fn new<C: Commit<T>>(commit: C) -> Self {
        let keys = commit.keys();

        let m_hasher = Self::gen_phf(keys);

        let entries = Self::gen_boxed_entries(&m_hasher, commit);

        Self { m_hasher, entries }
    }

    fn gen_boxed_entries<C: Commit<T>>(phf: &Mphf<AssetLocation>, commit: C) -> Box<[Entry<T>]> {
        let entries = commit.consume();

        let mut boxed = Box::<[Entry<T>]>::new_uninit_slice(entries.len());

        entries.into_iter().for_each(|entry| {
            let idx = phf.hash(entry.key()) as usize;

            boxed[idx].write(entry);
        });

        unsafe { boxed.assume_init() }
    }

    fn gen_phf(keys: Vec<AssetLocation>) -> Mphf<AssetLocation> {
        Mphf::new(Self::BASE, keys.as_slice())
    }
}

impl<T: Recordable> Record<T> for MappedRecord<T> {
    #[inline]
    fn get_by_key(&self, key: &AssetLocation) -> Option<&T> {
        let idx = self.m_hasher.try_hash(key)?;

        self.entries.get(idx as usize).and_then(|entry| {
            if entry.key() == key {
                return Some(entry.val());
            }

            None
        })
    }

    #[inline]
    fn get_by_id(&self, index: usize) -> Option<&T> {
        self.entries.get(index).map(|entry| entry.val())
    }

    #[inline]
    fn idx_to_key(&self, index: usize) -> Option<&AssetLocation> {
        self.entries.get(index).map(|entry| entry.key())
    }

    #[inline]
    fn key_to_idx(&self, key: &AssetLocation) -> Option<usize> {
        self.m_hasher.try_hash(key).map(|idx| idx as usize)
    }

    #[inline]
    fn keys(&self) -> Vec<&AssetLocation> {
        self.entries
            .iter()
            .map(|entry| entry.key())
            .collect::<Vec<_>>()
    }

    #[inline]
    fn len(&self) -> usize {
        self.entries.len()
    }
}
