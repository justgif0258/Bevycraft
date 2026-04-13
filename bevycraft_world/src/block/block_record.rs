use bevy::math::bounding::Aabb3d;
use bevy::prelude::Resource;
use boomphf::Mphf;
use bevycraft_core::prelude::{AssetLocation, Commit, Record};
use crate::block::block::BlockRef;
use crate::prelude::{Block, BlockDefinition};

#[derive(Resource)]
pub struct BlockRecord {
    hash_function       : Mphf<AssetLocation>,
    locations           : Box<[AssetLocation]>,
    definitions         : Box<[BlockDefinition]>,
    shape_descriptors   : Box<[ShapeDescriptor]>,
    shapes              : Box<[Aabb3d]>,
}

impl BlockRecord {
    #[inline(always)]
    pub fn get_ref_by_key(&self, key: &AssetLocation) -> Option<BlockRef<'_>> {
        Some(BlockRef {
            definition: self.get_definition_by_key(key)?,
            shapes: self.get_shapes_by_key(key)?,
        })
    }
    
    #[inline(always)]
    pub fn get_ref_by_idx(&self, idx: usize) -> Option<BlockRef<'_>> {
        Some(BlockRef {
            definition: self.get_definition_by_idx(idx)?,
            shapes: self.get_shapes_by_idx(idx)?,
        })
    }
    
    #[inline(always)]
    pub fn get_definition_by_key(&self, key: &AssetLocation) -> Option<&BlockDefinition> {
        let idx = self.hash_key(key)?;

        self.definitions.get(idx)
    }

    #[inline(always)]
    pub fn get_definition_by_idx(&self, idx: usize) -> Option<&BlockDefinition> {
        self.definitions.get(idx)
    }

    #[inline(always)]
    pub fn get_shapes_by_key(&self, key: &AssetLocation) -> Option<&[Aabb3d]> {
        let idx = self.hash_key(key)?;

        let descriptor = &self.shape_descriptors[idx];

        Some(&self.shapes[descriptor.start as usize..descriptor.length as usize])
    }

    #[inline(always)]
    pub fn get_shapes_by_idx(&self, idx: usize) -> Option<&[Aabb3d]> {
        let descriptor = &self.shape_descriptors[idx];

        Some(&self.shapes[descriptor.start as usize..descriptor.length as usize])
    }

    #[inline(always)]
    fn hash_key(&self, key: &AssetLocation) -> Option<usize> {
        let idx = self.hash_function.try_hash(key)? as usize;

        self.locations.get(idx)
            .and_then(|k| {
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
        let hash_function = Mphf::new(3.3f64, commit.cloned_keys().as_slice());

        let mut locations           = Box::<[AssetLocation]>::new_uninit_slice(size);
        let mut definitions         = Box::<[BlockDefinition]>::new_uninit_slice(size);
        let mut shape_descriptors   = Box::<[ShapeDescriptor]>::new_uninit_slice(size);
        let mut shapes: Vec<Aabb3d>                     = Vec::with_capacity(size);

        commit.into_iter()
            .for_each(|(key, block)| {
                let idx = hash_function.hash(&key) as usize;
                
                let shape_descriptor = ShapeDescriptor {
                    start: shapes.len() as u32,
                    length: block.shapes().len() as u32,
                };

                locations[idx].write(key);
                definitions[idx].write(block.definition().clone());
                shape_descriptors[idx].write(shape_descriptor);
                shapes.extend_from_slice(block.shapes());
            });

        unsafe {
            Self {
                hash_function,
                locations: locations.assume_init(),
                definitions: definitions.assume_init(),
                shape_descriptors: shape_descriptors.assume_init(),
                shapes: shapes.into_boxed_slice()
            }
        }
    }

    #[inline(always)]
    fn key_to_idx(&self, key: &AssetLocation) -> Option<usize> {
        self.hash_key(key)
    }

    #[inline(always)]
    fn idx_to_key(&self, id: usize) -> Option<&AssetLocation> {
        self.locations.get(id).map(|key| key)
    }

    #[inline(always)]
    fn keys(&self) -> Vec<&AssetLocation> {
        self.locations
            .iter()
            .map(|key| key)
            .collect()
    }

    #[inline(always)]
    fn len(&self) -> usize {
        self.locations.len()
    }
}

struct ShapeDescriptor {
    start   : u32,
    length  : u32,
}