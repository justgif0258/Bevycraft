use std::cmp::Ordering;
use std::collections::{BinaryHeap, VecDeque};
use std::sync::Arc;
use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::Resource;
use bevy::tasks::Task;
use crate::prelude::{Chunk, ChunkPos};

#[derive(Resource)]
pub struct ChunkMap {
    pub(crate) chunks: HashMap<ChunkPos, Chunk>,

    pub(crate) pending: HashMap<ChunkPos, Task<Chunk>>,

    pub(crate) load_queue: BinaryHeap<LoadRequest>,

    pub(crate) inflight: HashSet<ChunkPos>,

    pub(crate) unload_queue: VecDeque<ChunkPos>,

    pub max_concurrent: usize,
}

impl ChunkMap {
    #[inline]
    pub const fn new(max_concurrent: usize) -> Self {
        Self {
            chunks: HashMap::new(),
            pending: HashMap::new(),
            load_queue: BinaryHeap::new(),
            inflight: HashSet::new(),
            unload_queue: VecDeque::new(),
            max_concurrent,
        }
    }

    #[inline]
    pub fn get(&self, pos: &ChunkPos) -> Option<&Chunk> {
        self.chunks.get(pos)
    }

    #[inline]
    pub fn get_mut(&mut self, pos: &ChunkPos) -> Option<&mut Chunk> {
        self.chunks.get_mut(pos)
    }

    #[inline]
    pub fn is_loaded(&self, pos: &ChunkPos) -> bool {
        self.chunks.contains_key(pos)
    }

    #[inline]
    pub fn is_inflight(&self, pos: &ChunkPos) -> bool {
        self.inflight.contains(pos)
    }

    #[inline]
    pub fn loaded_count(&self) -> usize {
        self.chunks.len()
    }

    #[inline]
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    #[inline]
    pub fn enqueue(&mut self, pos: ChunkPos, dist_sq: i32) {
        if self.chunks.contains_key(&pos) || !self.inflight.insert(pos) {
            return;
        }

        self.load_queue.push(LoadRequest { dist_sq, pos })
    }

    #[inline]
    pub fn enqueue_unload(&mut self, pos: ChunkPos) {
        if self.chunks.contains_key(&pos) {
            self.unload_queue.push_back(pos);
        }
    }

    #[inline]
    pub fn remove(&mut self, pos: &ChunkPos) -> Option<Chunk> {
        self.inflight.remove(pos);
        self.chunks.remove(pos)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct LoadRequest {
    pub dist_sq: i32,
    pub pos: ChunkPos,
}

impl Ord for LoadRequest {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> Ordering {
        other.dist_sq.cmp(&self.dist_sq)
    }
}

impl PartialOrd for LoadRequest {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Resource, Clone)]
pub struct GeneratorResource(pub Arc<dyn ChunkGenerator>);

impl GeneratorResource {
    pub fn new<G: ChunkGenerator>(generator: G) -> Self {
        Self(Arc::new(generator))
    }
}

pub trait ChunkGenerator: Send + Sync + 'static {
    fn generate(&self, chunk_pos: ChunkPos) -> Chunk;
}

#[derive(Resource)]
pub struct ChunkLoaderConfig {
    pub view_distance: i32,
    pub unload_margin: i32,
}

impl Default for ChunkLoaderConfig {
    fn default() -> Self {
        Self { view_distance: 8, unload_margin: 2 }
    }
}