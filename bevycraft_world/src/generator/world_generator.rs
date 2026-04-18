use std::sync::Arc;
use bevy::prelude::Resource;
use bevycraft_core::prelude::{AssetLocation, Record};
use crate::prelude::{BlockRecord, Chunk, SectionPool};

pub struct SuperflatGenerator(pub u32);

impl WorldGenerator for SuperflatGenerator {
    #[inline(always)]
    fn seed(&self) -> u32 {
        self.0
    }

    #[inline(always)]
    fn generate_base_terrain(
        &self,
        chunk: &mut Chunk,
        blocks: &BlockRecord,
    ) {
        let air = blocks.key_to_idx(&AssetLocation::with_default_namespace("air"))
            .unwrap() as u32;

        let bedrock_id = blocks.key_to_idx(&AssetLocation::with_default_namespace("bedrock"))
            .unwrap() as u32;

        let dirt_id = blocks.key_to_idx(&AssetLocation::with_default_namespace("dirt"))
            .unwrap() as u32;

        let grass_block_id = blocks.key_to_idx(&AssetLocation::with_default_namespace("grass_block"))
            .unwrap() as u32;

        for x in 0..16 {
            for z in 0..16 {
                chunk.set_at([x, 0, z], bedrock_id);

                for y in 1..=3 {
                    chunk.set_at([x, y, z], dirt_id);
                }

                chunk.set_at([x, 4, z], grass_block_id);

                for y in 5..16 {
                    chunk.set_at([x, y, z], air);
                }
            }
        }
    }
}

#[derive(Resource)]
pub struct ActiveWorldGenerator {
    pub generator: Arc<dyn WorldGenerator>,
}

impl ActiveWorldGenerator {
    #[inline(always)]
    pub fn new(generator: impl WorldGenerator) -> Self {
        Self { generator: Arc::new(generator) }
    }
}

pub trait WorldGenerator: Send + Sync + 'static {
    fn seed(&self) -> u32;

    fn generate_base_terrain(
        &self,
        chunk: &mut Chunk,
        blocks: &BlockRecord,
    );

    #[inline]
    #[allow(unused_variables)]
    fn generate_features(
        &self,
        chunk: &mut Chunk,
        blocks: &BlockRecord,
    ) { return; }
}