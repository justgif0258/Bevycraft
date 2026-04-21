use bevy::prelude::Resource;
use boomphf::Mphf;
use bevycraft_core::prelude::*;
use crate::prelude::*;

#[derive(Resource)]
pub struct BlockRecord {
    hash_function:  Mphf<AssetLocation>,
    blocks:         Box<[(AssetLocation, Block)]>,
}

impl BlockRecord {
    #[inline(always)]
    fn hash_key(&self, key: &AssetLocation) -> Option<usize> {
        let idx = self.hash_function.try_hash(key)? as usize;

        self.blocks.get(idx)
            .and_then(|(k, _)| {
                if k != key {
                    return None;
                }

                Some(idx)
            })
    }
}

impl Record for BlockRecord {
    type Value = Block;

    #[inline]
    fn finish<C>(commit: C) -> Self
    where
        C: Commit<Value=Self::Value>
    {
        let size = commit.len();
        let hash_function = Mphf::new(
            3.3f64, 
            commit.iter_keys()
                .cloned()
                .collect::<Vec<_>>()
                .as_slice()
        );

        let mut blocks = Box::<[(AssetLocation, Block)]>::new_uninit_slice(size);

        commit.into_iter()
            .for_each(|entry| {
                let idx = hash_function.hash(&entry.0) as usize;

                blocks[idx].write(entry);
            });

        unsafe {
            Self {
                hash_function,
                blocks: blocks.assume_init(),
            }
        }
    }

    #[inline(always)]
    fn get_by_key(&self, key: &AssetLocation) -> Option<&Self::Value> {
        let idx = self.hash_key(key)?;
        
        self.blocks.get(idx)
            .and_then(|(k, b)| {
                if k != key {
                    return None;
                }
                
                Some(b)
            })
    }

    #[inline(always)]
    fn get_by_idx(&self, idx: usize) -> Option<&Self::Value> {
        self.blocks.get(idx).map(|(_, b)| b)
    }

    #[inline(always)]
    fn key_to_idx(&self, key: &AssetLocation) -> Option<usize> {
        self.hash_key(key)
    }

    #[inline(always)]
    fn idx_to_key(&self, id: usize) -> Option<&AssetLocation> {
        self.blocks.get(id).map(|(key, _)| key)
    }

    #[inline(always)]
    fn iter(&self) -> impl Iterator<Item = &(AssetLocation, Self::Value)> {
        self.blocks.iter()
    }

    #[inline(always)]
    fn iter_keys(&self) -> impl Iterator<Item = &AssetLocation> {
        self.blocks
            .iter()
            .map(|(key, _)| key)
    }

    #[inline(always)]
    fn len(&self) -> usize {
        self.blocks.len()
    }
}