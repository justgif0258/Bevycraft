use rayon::iter::{ParallelBridge, ParallelIterator};
use bevy::prelude::{info, Resource, Vec3};
use bevy::tasks::AsyncComputeTaskPool;
use crossbeam::channel::{Receiver, Sender};
use dashmap::DashMap;
use rapidhash::fast::RandomState;
use rayon::iter::IntoParallelRefIterator;
use crate::generator::chunk_generator::ChunkSource;
use crate::prelude::{BlockRecord, Chunk, ChunkGenerator, ChunkPos};
use crate::prelude::ChunkState::PendingSource;

#[derive(Resource)]
pub struct SparseSpatialMap {
    chunks: DashMap<ChunkPos, ChunkState, RandomState>,

    event_rx: Receiver<(ChunkPos, ChunkEvent)>,
    event_tx: Sender<(ChunkPos, ChunkEvent)>,

    source: ChunkSource,
    blocks: BlockRecord,

    radius: i32,
}

impl SparseSpatialMap {
    #[inline]
    pub fn new(generator: impl ChunkGenerator, blocks: BlockRecord, radius: i32) -> Self {
        assert!(radius > 0, "Radius must be greater than 0");

        let (event_tx, event_rx) = crossbeam::channel::unbounded();

        let source = ChunkSource::new(generator);

        Self {
            chunks: DashMap::with_hasher(RandomState::new()),

            event_rx,
            event_tx,

            source,
            blocks,

            radius,
        }
    }

    #[inline]
    pub fn handle_chunk_event_queue(&mut self) {
        self.event_rx.try_iter()
            .par_bridge()
            .for_each(|(pos, event)| {
                match event {
                    ChunkEvent::ChunkLoaded(chunk) => {
                        self.chunks.insert(pos, ChunkState::Loaded(chunk));

                        info!("Successfully loaded chunk {:?}", pos);
                    }
                    ChunkEvent::ChunkUnloaded => { 
                        self.chunks.remove(&pos); 
                    
                        info!("Successfully unloaded chunk {:?}", pos);
                    },
                    ChunkEvent::ChunkEdited => {}
                }
            })
    }

    #[inline]
    pub fn insert_chunk_at(&mut self, chunk_pos: ChunkPos, chunk: Chunk) {
        self.chunks.insert(chunk_pos, ChunkState::Loaded(chunk));
    }

    #[inline]
    pub fn queue_chunks_for_generation(&self, center: impl Into<Vec3>, pool: &AsyncComputeTaskPool) {
        let center_chunk = ChunkPos::from_world_pos(center);

        let radius = self.radius;
        let radius_squared = radius * radius;

        for x in (center_chunk.x - radius)..(center_chunk.x + radius) {
            for y in (center_chunk.y - radius)..(center_chunk.y + radius) {
                for z in (center_chunk.z - radius)..center_chunk.z {
                    let current = ChunkPos::new(x, y, z);

                    if current.distance_squared(center_chunk) > radius_squared {
                        continue;
                    }

                    if self.chunk_has_state_at(&current) {
                        continue;
                    }

                    self.chunks.insert(current, PendingSource);

                    let source = self.source.clone();

                    let blocks = self.blocks.clone();

                    let tx = self.event_tx.clone();

                    pool.spawn(async move {
                        let chunk = Chunk::new_from_source(source, current, blocks);

                        tx.send((current, ChunkEvent::ChunkLoaded(chunk)))
                    }).detach();
                }
            }
        }
    }

    #[inline]
    pub fn queue_chunks_for_unloading(&self, center: impl Into<Vec3>) {
        let center_chunk = ChunkPos::from_world_pos(center);

        let radius_squared = self.radius * self.radius;

        let tx = self.event_tx.clone();

        self.chunks
            .par_iter()
            .for_each(|chunk| {
                let pos = *chunk.key();

                let dist_sq = pos.distance_squared(center_chunk);

                if dist_sq > radius_squared {
                    tx.send((pos, ChunkEvent::ChunkUnloaded))
                        .unwrap();
                }
            })
    }

    #[inline]
    pub fn handle_event_queue<F>(&self, mut f: F)
    where
        F: FnMut(ChunkPos, ChunkEvent),
    {
        self.event_rx.try_iter()
            .for_each(|(pos, event)| f(pos, event))
    }

    #[inline(always)]
    pub fn is_chunk_loaded_at(&self, position: &ChunkPos) -> bool {
        if let Some(state) = self.chunks.get(position) {
            return state.value().is_loaded()
        }

        false
    }

    #[inline(always)]
    pub fn chunk_has_state_at(&self, position: &ChunkPos) -> bool {
        self.chunks.contains_key(position)
    }
}

pub enum ChunkEvent {
    ChunkLoaded(Chunk),
    ChunkUnloaded,
    ChunkEdited,
}

pub enum ChunkState {
    PendingSource,
    Loaded(Chunk),
}

impl ChunkState {
    #[inline(always)]
    pub const fn is_loaded(&self) -> bool {
        match self {
            ChunkState::Loaded(_) => true,
            _ => false,
        }
    }
}