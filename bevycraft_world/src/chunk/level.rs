use std::sync::Arc;
use bevy::platform::hash::NoOpHash;
use bevy::prelude::{IVec3, Resource, Vec3};
use bevy::tasks::AsyncComputeTaskPool;
use crossbeam::channel::{unbounded, Receiver, Sender};
use dashmap::*;
use crate::chunk::level::ChunkState::{Loaded, Pending};
use crate::prelude::*;

#[derive(Resource)]
pub struct Level {
    chunks: Arc<DashMap<ChunkPos, ChunkState, NoOpHash>>,
    dirty:  DashSet<ChunkPos, NoOpHash>,

    gen_tx: Sender<(ChunkPos, Chunk)>,
    gen_rx: Receiver<(ChunkPos, Chunk)>,

    blocks:     Arc<BlockRecord>,
    generator:  ActiveWorldGenerator,

    render_distance: i32,
}

impl Level {
    #[inline]
    pub fn new(
        blocks: Arc<BlockRecord>,
        generator: impl WorldGenerator,
        render_distance: i32,
    ) -> Self {
        assert!(render_distance > 0, "Render distance must be greater than 0");

        let (gen_tx, gen_rx) = unbounded::<(ChunkPos, Chunk)>();

        Self {
            chunks: Arc::new(DashMap::with_hasher(NoOpHash)),
            dirty: DashSet::with_hasher(NoOpHash),

            gen_tx,
            gen_rx,

            blocks,
            generator: ActiveWorldGenerator::new(generator),

            render_distance,
        }
    }
    
    #[inline]
    pub fn set_at(
        &self,
        pos: IVec3,
        block_type: BlockType,
    ) {
        let target_chunk = ChunkPos::from_world_pos(pos.as_vec3());
        
        if !self.chunks.contains_key(&target_chunk) { 
            return;
        }
        
        self.chunks.entry(target_chunk)
            .and_modify(|state| {
                if let Loaded(chunk) = state {
                    chunk.set_at(pos, block_type);
                    
                    self.dirty.insert(target_chunk);
                }
            });
    }
    
    #[inline]
    pub fn get_at(
        &self,
        pos: IVec3
    ) -> Option<BlockType> {
        let target_chunk = ChunkPos::from_world_pos(pos.as_vec3());
        
        if !self.chunks.contains_key(&target_chunk) { 
            return None;
        }
        
        self.chunks.get(&target_chunk)
            .and_then(|chunk| {
                if let Loaded(chunk) = chunk.value() {
                    let local_pos = world_to_local(pos);
                    
                    return Some(chunk.get_at(local_pos))
                }
                
                None
            })
    }

    #[inline]
    pub fn load_in_range_if_not(
        &self,
        center: Vec3,
        pool: &AsyncComputeTaskPool
    ) {
        let center_chunk = ChunkPos::from_world_pos(center);

        for x in -self.render_distance..self.render_distance {
            for z in -self.render_distance..self.render_distance {
                let current = ChunkPos::new(
                    center_chunk.0.x + x,
                    center_chunk.0.y + z,
                );

                self.try_queue_chunk(current, pool);
            }
        }
    }
    
    #[inline]
    pub fn load_outside_of_range(
        &self,
        center: Vec3,
    ) {
        let center_chunk = ChunkPos::from_world_pos(center);
        
        self.chunks
            .retain(|pos, _| {
                let current_pos = pos.0;

                let dist_x = (center_chunk.0.x - current_pos.x).abs();
                let dist_z = (center_chunk.0.y - current_pos.y).abs();

                if dist_x > self.render_distance || dist_z > self.render_distance {
                    return false;
                }
                
                return true;
            })
    }

    #[inline]
    pub fn handle_tasks(&self) {
        self.gen_rx.try_iter()
            .for_each(|(chunk_pos, chunk)| {
                self.chunks.entry(chunk_pos)
                    .and_modify(|state| {
                        *state = Loaded(chunk);
                    });
            })
    }

    #[inline]
    pub fn try_queue_chunk(
        &self,
        pos: ChunkPos,
        pool: &AsyncComputeTaskPool,
    ) {
        if self.chunks.contains_key(&pos) {
            return;
        }

        self.chunks.insert(pos, Pending);

        let blocks = self.blocks.clone();
        let generator = self.generator.clone();

        let chunks = self.chunks.clone();

        pool.spawn(async move {
            let chunk = Chunk::generate_using(
                pos,
                blocks,
                generator,
            );

            chunks.entry(pos)
                .and_modify(|state| {
                    *state = Loaded(chunk);
                });
        }).detach();
    }
}

pub enum ChunkState {
    Pending,
    Loaded(Chunk),
}

#[inline(always)]
fn world_to_local(pos: impl Into<IVec3>) -> IVec3 {
    let pos = pos.into();
    
    IVec3::new(
        pos.x.rem_euclid(SECTION_SIZE),
        pos.y,
        pos.z.rem_euclid(SECTION_SIZE),
    )
}