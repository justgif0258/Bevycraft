use bevy::{
    ecs::resource::Resource,
    platform::{collections::HashMap, hash::NoOpHash},
};
use bevycraft_core::prelude::BlockType;

use crate::model::block_model::BlockModel;

#[derive(Resource, Default)]
pub struct ModelCache {
    blocks: HashMap<BlockType, BlockModel, NoOpHash>,
}

impl ModelCache {
    pub fn new(blocks: HashMap<BlockType, BlockModel, NoOpHash>) -> Self {
        Self { blocks }
    }

    #[inline]
    pub fn get(&self, block_type: BlockType) -> Option<&BlockModel> {
        self.blocks.get(&block_type)
    }
}
