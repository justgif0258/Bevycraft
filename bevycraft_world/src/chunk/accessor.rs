use std::mem::{transmute, transmute_copy};
use bevy::ecs::world::CommandQueue;
use bevy::platform::collections::HashMap;
use bevy::platform::hash::NoOpHash;
use bevy::prelude::*;
use bevy::tasks::Task;
use crate::prelude::{ActiveWorldGenerator, Chunk, WorldGenerator};

#[derive(Resource)]
pub struct ChunkAccessor {
    active: HashMap<u64, Entity, NoOpHash>,

    generator: ActiveWorldGenerator,

    pub view_distance: i32,
}

impl ChunkAccessor {
    #[inline]
    #[must_use]
    pub fn new(
        view_distance: i32,
        generator: impl WorldGenerator,
    ) -> Self {
        assert!(view_distance > 0, "View distance must be greater than 0");

        Self {
            active: HashMap::with_hasher(NoOpHash),
            generator: ActiveWorldGenerator::new(generator),
            view_distance,
        }
    }
    
    #[inline(always)]
    pub fn get_chunk_entity(&self, position: &IVec2) -> Option<Entity> {
        self.active.get(&hash_position_copy(position)).copied()
    }

    #[inline(always)]
    pub fn insert_chunk_entity(&mut self, position: IVec2, entity: Entity) {
        self.active.insert(
            hash_position(position),
            entity,
        );
    }

    #[inline(always)]
    pub fn unload_chunks_out_of_range(
        &mut self,
        center: IVec2,
        world: &World,
    ) -> CommandQueue {
        let mut queue = CommandQueue::default();
        let mut commands = Commands::new(&mut queue, world);

        self.active.retain(|hash, entity| {
            let chunk_pos: IVec2 = position_from_hash_copy(hash);

            let dist_x = (chunk_pos.x - center.x).abs();
            let dist_z = (chunk_pos.y - center.y).abs();

            if dist_x > self.view_distance || dist_z > self.view_distance {
                commands.entity(*entity).despawn();

                return false;
            }

            return true;
        });

        queue
    }
    
    #[inline(always)]
    pub fn is_loaded(&self, position: &IVec2) -> bool {
        self.active.contains_key(hash_position_ref(position))
    }
    
    #[inline(always)]
    pub fn get_generator(&self) -> ActiveWorldGenerator {
        self.generator.clone()
    }
}

#[derive(Component)]
pub enum ChunkState {
    Unloaded,
    Generating(Task<Chunk>),
    Loaded(Chunk),
}

#[inline(always)]
const fn position_from_hash_copy(hash: &u64) -> IVec2 {
    unsafe { transmute_copy(hash) }
}

#[inline(always)]
const fn hash_position_copy(pos: &IVec2) -> u64 {
    unsafe { transmute_copy(pos) }
}

#[inline(always)]
const fn hash_position(pos: IVec2) -> u64 {
    unsafe { transmute(pos) }
}

#[inline(always)]
const fn hash_position_ref(pos: &IVec2) -> &u64 {
    unsafe { transmute(pos) }
}