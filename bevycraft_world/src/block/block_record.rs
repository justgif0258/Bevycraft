use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::mem::{transmute, transmute_copy};
use std::num::NonZeroU32;
use std::sync::{Arc, LazyLock};
use bevy::prelude::Resource;
use boomphf::Mphf;
use voxelis::VoxelTrait;
use bevycraft_core::prelude::*;
use crate::block::block_record::BlockType::Id;
use crate::prelude::*;

static AIR_LOCATION: LazyLock<AssetLocation> = LazyLock::new(|| AssetLocation::with_default_namespace("air"));

#[derive(Resource)]
pub struct BlockRecord(Arc<BlockRecordInner>);

struct BlockRecordInner {
    hash_function:  Mphf<AssetLocation>,
    blocks:         Box<[(AssetLocation, Block)]>,
}

impl Clone for BlockRecord {
    #[inline(always)]
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl Record for BlockRecord {
    type Value = Block;

    type Index = BlockType;

    #[inline]
    fn finish<C>(commit: C) -> Self
    where
        C: Commit<Value=Self::Value>
    {
        let size = commit.len() + 1;
        let hash_function = Mphf::new(
            3.3f64, 
            commit.iter_keys()
                .cloned()
                .collect::<Vec<_>>()
                .as_slice()
        );

        let mut blocks = Box::<[(AssetLocation, Block)]>::new_uninit_slice(size);

        blocks[0].write((AIR_LOCATION.clone(), Block::default()));

        commit.into_iter()
            .for_each(|entry| {
                let idx = hash_key(&hash_function, &entry.0)
                    .unwrap();

                blocks[idx].write(entry);
            });

        unsafe {
            Self(Arc::new(BlockRecordInner {
                hash_function,
                blocks: blocks.assume_init()
            }))
        }
    }

    #[inline(always)]
    fn get_by_key(&self, key: &AssetLocation) -> Option<&Self::Value> {
        if key.eq(&AIR_LOCATION) {
            return Some(&self.0.blocks[0].1)
        }

        let idx = hash_key(&self.0.hash_function, key)?;
        
        self.0.blocks.get(idx)
            .and_then(|(k, b)| {
                if k != key {
                    return None;
                }
                
                Some(b)
            })
    }

    #[inline(always)]
    fn get_by_idx(&self, index: Self::Index) -> Option<&Self::Value> {
        self.0.blocks.get(index.inner() as usize)
            .map(|(_, b)| b)
    }

    #[inline(always)]
    fn key_to_idx(&self, key: &AssetLocation) -> Option<Self::Index> {
        if key.eq(&AIR_LOCATION) {
            return Some(BlockType::Air)
        }
        
        let idx = hash_key(&self.0.hash_function, key)?;
        
        if &self.0.blocks[idx].0 != key { 
            return None;
        }

        Some(BlockType::from(idx as u32))
    }

    #[inline(always)]
    fn idx_to_key(&self, index: Self::Index) -> Option<&AssetLocation> {
        match index {
            BlockType::Air => Some(&AIR_LOCATION),
            Id(idx) => {
                self.0.blocks
                    .get(idx.get() as usize)
                    .map(|(key, _)| key)
            }
        }
    }

    #[inline(always)]
    fn iter(&self) -> impl Iterator<Item = &(AssetLocation, Self::Value)> {
        self.0.blocks.iter()
    }

    #[inline(always)]
    fn iter_keys(&self) -> impl Iterator<Item = &AssetLocation> {
        self.0.blocks
            .iter()
            .map(|(key, _)| key)
    }

    #[inline(always)]
    fn len(&self) -> usize {
        self.0.blocks.len()
    }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum BlockType {
    Air,
    Id(NonZeroU32)
}

impl VoxelTrait for BlockType {}

impl Default for BlockType {
    #[inline(always)]
    fn default() -> Self {
        BlockType::Air
    }
}

impl BlockType {
    #[inline(always)]
    pub const fn from(value: u32) -> Self {
        unsafe { transmute(value) }
    }

    #[inline(always)]
    pub const fn inner(self) -> u32 {
        unsafe { transmute(self) }
    }
    
    #[inline(always)]
    pub fn is_air(&self) -> bool {
        *self == BlockType::Air
    }
}

impl Hash for BlockType {
    #[inline(always)]
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(unsafe { transmute_copy::<_, u32>(self) as u64 })
    }
}

impl Display for BlockType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockType::Air => f.write_str("BlockType(Air)"),
            Id(i) => write!(f, "BlockType(Id = {})", i.get()),
        }
    }
}

#[inline(always)]
fn hash_key(function: &Mphf<AssetLocation>, key: &AssetLocation) -> Option<usize> {
    Some(function.try_hash(key)? as usize + 1)
}