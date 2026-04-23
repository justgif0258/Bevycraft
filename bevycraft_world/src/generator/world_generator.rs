use std::sync::Arc;
use bevy::prelude::Resource;
use bevycraft_core::prelude::{AssetLocation, Record};
use crate::prelude::{BlockRecord, Chunk, ChunkPos};

pub struct SuperflatGenerator(pub u32);

impl WorldGenerator for SuperflatGenerator {
    #[inline(always)]
    fn seed(&self) -> u32 {
        self.0
    }

    #[inline(always)]
    fn generate_base_terrain(
        &self,
        _position: ChunkPos,
        chunk   : &mut Chunk,
        blocks  : Arc<BlockRecord>,
    ) {
        let air = blocks.key_to_idx(&AssetLocation::with_default_namespace("air"))
            .unwrap();

        let bedrock_id = blocks.key_to_idx(&AssetLocation::with_default_namespace("bedrock"))
            .unwrap();

        let dirt_id = blocks.key_to_idx(&AssetLocation::with_default_namespace("dirt"))
            .unwrap();

        let grass_block_id = blocks.key_to_idx(&AssetLocation::with_default_namespace("grass_block"))
            .unwrap();

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

impl Clone for ActiveWorldGenerator {
    #[inline(always)]
    fn clone(&self) -> Self {
        Self { generator: self.generator.clone() }
    }
}

impl ActiveWorldGenerator {
    #[inline(always)]
    pub fn new(generator: impl WorldGenerator) -> Self {
        Self { generator: Arc::new(generator) }
    }
    
    #[inline(always)]
    pub fn generate_chunk(
        &self,
        position: ChunkPos,
        chunk   : &mut Chunk,
        blocks  : Arc<BlockRecord>,
    ) {
        self.generator.generate_base_terrain(position, chunk, blocks.clone());
        self.generator.carve_terrain(position, chunk);
        self.generator.generate_features(position, chunk, blocks);
    }
}

impl WorldGenerator for ActiveWorldGenerator {
    #[inline(always)]
    fn seed(&self) -> u32 { self.generator.seed() }

    #[inline(always)]
    fn generate_base_terrain(
        &self,
        position: ChunkPos,
        chunk   : &mut Chunk,
        blocks  : Arc<BlockRecord>,
    ) { self.generator.generate_base_terrain(position, chunk, blocks); }

    #[inline(always)]
    fn carve_terrain(
        &self, 
        position: ChunkPos, 
        chunk   : &mut Chunk
    ) { self.generator.carve_terrain(position, chunk); }

    #[inline(always)]
    fn generate_features(
        &self, 
        position: ChunkPos,
        chunk   : &mut Chunk, 
        blocks  : Arc<BlockRecord>
    ) { self.generator.generate_features(position, chunk, blocks); }
}

pub trait WorldGenerator: Send + Sync + 'static {
    fn seed(&self) -> u32;

    fn generate_base_terrain(
        &self,
        position: ChunkPos,
        chunk   : &mut Chunk,
        blocks  : Arc<BlockRecord>,
    );

    #[inline]
    #[allow(unused_variables)]
    fn carve_terrain(
        &self,
        position: ChunkPos,
        chunk   : &mut Chunk,
    ) { return; }

    #[inline]
    #[allow(unused_variables)]
    fn generate_features(
        &self,
        position: ChunkPos,
        chunk   : &mut Chunk,
        blocks  : Arc<BlockRecord>,
    ) { return; }
}