use crate::mesh::chunk::ChunkMeshOutput;
use crate::prelude::RenderMode;
use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::{Component, Entity, Resource};
use bevy::tasks::Task;
use bevycraft_world::prelude::ChunkPos;

#[derive(Component)]
pub struct ChunkMeshRoot(pub ChunkPos);

#[derive(Component)]
pub struct ChunkMeshLayer(pub RenderMode);

#[derive(Default, Resource)]
pub struct ChunkEntityMap(pub HashMap<ChunkPos, Entity>);

pub struct BatchOutput {
    pub results: Vec<(ChunkPos, ChunkMeshOutput)>,
}

pub struct InflightBatch {
    pub task: Task<BatchOutput>,
}

#[derive(Resource)]
pub struct MeshingQueue {
    pub pending: HashSet<ChunkPos>,
    pub inflight_batches: Vec<InflightBatch>,
    pub inflight_chunks: HashSet<ChunkPos>,
    pub budget: usize,
}

impl Default for MeshingQueue {
    fn default() -> Self {
        Self {
            pending: HashSet::default(),
            inflight_batches: Vec::new(),
            inflight_chunks: HashSet::default(),
            budget: 64,
        }
    }
}