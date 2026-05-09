use bevy::{
    ecs::resource::Resource,
    platform::{collections::HashMap, hash::NoOpHash},
};
use bevycraft_core::prelude::{Block, Holder};

use crate::model::block_model::BlockModel;

#[derive(Resource, Default)]
pub struct ModelCache {
    blocks: HashMap<u64, BlockModel, NoOpHash>,
}

impl ModelCache {
    pub fn new(blocks: HashMap<u64, BlockModel, NoOpHash>) -> Self {
        Self { blocks }
    }

    #[inline]
    pub fn get(&self, block: Holder<'_, Block>) -> Option<&BlockModel> {
        self.blocks.get(&(*block as u64))
    }
}
