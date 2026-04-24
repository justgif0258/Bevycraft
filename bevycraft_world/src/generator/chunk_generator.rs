use std::sync::Arc;
use bevy::prelude::Resource;
use crate::prelude::{BlockRecord, Chunk, ChunkPos};

#[derive(Resource)]
pub struct ChunkSource {
    generator: Arc<dyn ChunkGenerator>,
}

impl ChunkSource {
    #[inline]
    pub fn new(generator: impl ChunkGenerator) -> Self {
        Self {
            generator: Arc::new(generator),
        }
    }
}

impl Clone for ChunkSource {
    #[inline(always)]
    fn clone(&self) -> Self {
        Self {
            generator: self.generator.clone(),
        }
    }
}

impl ChunkGenerator for ChunkSource {
    #[inline(always)]
    fn seed(&self) -> i32 {
        self.generator.seed()
    }
    
    #[inline(always)]
    fn fill(&self, position: ChunkPos, chunk: &mut Chunk, blocks: BlockRecord) {
        self.generator.fill(position, chunk, blocks);
    }

    #[inline(always)]
    fn carve(&self, position: ChunkPos, chunk: &mut Chunk) {
        self.generator.carve(position, chunk);
    }

    #[inline(always)]
    fn place_features(&self, position: ChunkPos, chunk: &mut Chunk, blocks: BlockRecord) {
        self.generator.place_features(position, chunk, blocks);
    }
}

pub trait ChunkGenerator: Send + Sync + 'static {
    fn seed(&self) -> i32;
    
    fn fill(
        &self,
        position: ChunkPos,
        chunk: &mut Chunk,
        blocks: BlockRecord,
    );
    
    #[allow(unused_variables)]
    fn carve(
        &self,
        position: ChunkPos,
        chunk: &mut Chunk,
    ) { return; }
    
    #[allow(unused_variables)]
    fn place_features(
        &self,
        position: ChunkPos,
        chunk: &mut Chunk,
        blocks: BlockRecord,
    ) { return; }
}