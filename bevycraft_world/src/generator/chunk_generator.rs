use std::sync::Arc;

use bevy::ecs::resource::Resource;

use crate::prelude::{Chunk, ChunkPos};

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
    fn fill(&self, position: ChunkPos, chunk: &mut Chunk) {
        self.generator.fill(position, chunk);
    }

    #[inline(always)]
    fn carve(&self, position: ChunkPos, chunk: &mut Chunk) {
        self.generator.carve(position, chunk);
    }

    #[inline(always)]
    fn place_features(&self, position: ChunkPos, chunk: &mut Chunk) {
        self.generator.place_features(position, chunk);
    }
}

pub trait ChunkGenerator: Send + Sync + 'static {
    fn seed(&self) -> i32;

    fn fill(&self, position: ChunkPos, chunk: &mut Chunk);

    #[allow(unused_variables)]
    fn carve(&self, position: ChunkPos, chunk: &mut Chunk) {
        return;
    }

    #[allow(unused_variables)]
    fn place_features(&self, position: ChunkPos, chunk: &mut Chunk) {
        return;
    }
}
