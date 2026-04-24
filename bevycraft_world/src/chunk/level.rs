use std::sync::Arc;
use bevy::platform::collections::{HashMap, HashSet};
use bevy::platform::hash::NoOpHash;
use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use crossbeam::channel::{unbounded, Receiver, Sender};
use crossbeam::queue::SegQueue;
use crate::prelude::*;

#[derive(Resource)]
pub struct Level {
    chunks: HashMap<ChunkPos, ChunkState, NoOpHash>,
    dirty:  HashSet<ChunkPos, NoOpHash>,

    unloaded_queue: SegQueue<(ChunkPos, Chunk)>,

    gen_rx: Receiver<(ChunkPos, Chunk)>,
    gen_tx: Sender<(ChunkPos, Chunk)>,

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

        let (gen_tx, gen_rx) = unbounded();

        Self {
            chunks: HashMap::with_hasher(NoOpHash),
            dirty: HashSet::with_hasher(NoOpHash),

            unloaded_queue: SegQueue::new(),

            gen_rx,
            gen_tx,

            blocks,
            generator: ActiveWorldGenerator::new(generator),

            render_distance,
        }
    }
    
    #[inline]
    pub fn set_at(
        &mut self,
        pos: IVec3,
        block_type: BlockType,
    ) {
        let target_chunk = ChunkPos::from_world_pos(pos.as_vec3());
        
        if !self.chunks.contains_key(&target_chunk) { 
            return;
        }
        
        self.chunks.entry(target_chunk)
            .and_modify(|state| {
                if let ChunkState::Loaded(chunk) = state {
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
                if let ChunkState::Loaded(chunk) = chunk {
                    let local_pos = world_to_local(pos);
                    
                    return Some(chunk.get_at(local_pos))
                }

                None
            })
    }
    
    #[inline]
    pub fn get_chunk_state(&self, pos: &ChunkPos) -> Option<&ChunkState> {
        self.chunks.get(pos)
    }

    #[inline]
    pub fn handle_chunk_task_queue(&mut self) {
        self.gen_rx
            .try_iter()
            .for_each(|(pos, chunk)| {
                self.chunks.entry(pos)
                    .and_modify(|state| {
                        *state = ChunkState::Loaded(chunk);
                    });

                self.dirty.insert(pos);

                for neighbor in pos.neighbors() {
                    if self.chunks.contains_key(&neighbor) {
                        self.dirty.insert(neighbor);
                    }
                }
            })
    }

    #[inline]
    pub fn queue_chunks_for_loading(
        &mut self,
        center: Vec3,
        pool: &AsyncComputeTaskPool
    ) {
        let center_chunk = ChunkPos::from_world_pos(center);

        let render_dist = self.render_distance;
        let render_dist_sq = self.render_distance_squared();

        for x in (center_chunk.0.x - render_dist)..(center_chunk.0.x + render_dist) {
            for z in (center_chunk.0.y - render_dist)..(center_chunk.0.y + render_dist) {
                let current = ChunkPos::new(x, z);

                if center_chunk.distance_squared(current) <= render_dist_sq {

                    self.try_queue_chunk_task(current, pool);
                }
            }
        }
    }
    
    #[inline]
    pub fn queue_chunks_for_unloading(
        &mut self,
        center: Vec3,
    ) {
        let center_chunk = ChunkPos::from_world_pos(center);
        let render_dist = self.render_distance_squared();

        self.chunks
            .extract_if(|&pos, _| center_chunk.distance_squared(pos) > render_dist)
            .for_each(|(pos, state)| {
                if let ChunkState::Loaded(chunk) = state {
                    self.unloaded_queue.push((pos, chunk));
                }
            });
    }

    #[inline]
    pub fn try_queue_chunk_task(
        &mut self,
        pos: ChunkPos,
        pool: &AsyncComputeTaskPool,
    ) {
        if self.chunks.contains_key(&pos) {
            return;
        }

        self.chunks.insert(pos, ChunkState::Pending);

        let blocks = self.blocks.clone();
        let generator = self.generator.clone();

        let tx = self.gen_tx.clone();

        pool.spawn(async move {
            let chunk = Chunk::generate_using(
                pos,
                blocks,
                generator,
            );

            tx.send((pos, chunk))
                .expect("Failed to send chunk");
        }).detach();
    }

    #[inline]
    pub fn try_handle_unloaded_chunks<F>(&self, mut f: F)
    where
        F: FnMut(ChunkPos, Chunk),
    {
        while let Some((pos, chunk)) = self.unloaded_queue.pop() {
            f(pos, chunk);
        }
    }

    #[inline(always)]
    const fn render_distance_squared(&self) -> i32 {
        self.render_distance * self.render_distance
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