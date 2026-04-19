use std::mem::{transmute, transmute_copy};
use std::sync::Arc;
use bevy::platform::collections::HashMap;
use bevy::platform::hash::NoOpHash;
use bevy::prelude::*;
use crate::prelude::{ActiveWorldGenerator, WorldGenerator};

#[derive(Resource)]
pub struct ChunkManager {
    pub active: HashMap<u64, Entity, NoOpHash>,

    pub view_distance: u32,

    generator: ActiveWorldGenerator,
}

impl ChunkManager {
    #[inline]
    #[must_use]
    pub fn new(
        view_distance: u32,
        generator: impl WorldGenerator,
    ) -> Self {
        Self {
            active: HashMap::with_hasher(NoOpHash),
            view_distance,
            generator: ActiveWorldGenerator::new(generator),
        }
    }
    
    #[inline(always)]
    pub fn get_chunk_entity(&self, position: &IVec2) -> Option<&Entity> {
        self.active.get(&hash_position(*position))
    }

    #[inline]
    pub fn insert_chunk_entity(&mut self, position: IVec2, entity: Entity) {
        self.active.insert(
            hash_position(position),
            entity,
        );
    }
    
    #[inline(always)]
    pub fn is_loaded(&self, position: &IVec2) -> bool {
        self.active.contains_key(&hash_position(*position))
    }
    
    #[inline(always)]
    pub fn get_generator(&self) -> ActiveWorldGenerator {
        self.generator.clone()
    }
}

#[derive(Component)]
pub struct ChunkPos(pub IVec2);

#[derive(Component, Eq, PartialEq)]
pub enum ChunkState {
    WaitingForData,
    WaitingForNeighbors,
    WaitingForMesh,
    Ready
}

#[inline(always)]
const fn hash_position(pos: IVec2) -> u64 {
    unsafe { transmute(pos) }
}