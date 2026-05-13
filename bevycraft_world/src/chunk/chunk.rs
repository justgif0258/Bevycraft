use std::{
    fmt::{Display, Formatter, Result},
    hash::{Hash, Hasher},
    mem::transmute,
    ops::Sub,
};

use bevy::{
    ecs::component::Component,
    math::{IVec3, Vec3, bounding::Aabb3d},
};
use bevycraft_core::blocks::AIR;

use crate::prelude::*;

pub const CHUNK_SIZE: i32 = 16;

pub const CHUNK_LEN: usize = (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as usize;

#[derive(Component)]
pub struct Chunk {
    storage: ChunkStorage,

    pub dirty: bool,
}

impl Chunk {
    #[inline]
    pub fn empty() -> Self {
        Self {
            storage: ChunkStorage::Empty,
            dirty: false,
        }
    }

    #[inline]
    pub fn uniform(block: usize) -> Self {
        Self {
            storage: ChunkStorage::Single(block),
            dirty: false,
        }
    }

    #[inline]
    pub fn new_from_source(source: ChunkSource, position: ChunkPos) -> Self {
        let mut chunk = Chunk {
            storage: ChunkStorage::empty_pattern(),
            dirty: false,
        };

        source.fill(position, &mut chunk);
        source.carve(position, &mut chunk);
        source.place_features(position, &mut chunk);

        // We wanna make sure that the delivered Chunk is fully compressed
        // Generators tend to be messy sometimes
        chunk.storage.optimize();

        chunk
    }

    #[inline]
    pub fn set(&mut self, position: impl Into<IVec3>, block: usize) {
        let position = position.into();

        if position.cmplt(IVec3::ZERO).any() {
            return;
        }

        if position.cmpge(IVec3::splat(CHUNK_SIZE)).any() {
            return;
        }

        self.storage.set(position, block.into());
    }

    #[inline]
    pub fn remove(&mut self, position: impl Into<IVec3>) -> Option<usize> {
        let position = position.into();

        if position.cmplt(IVec3::ZERO).any() {
            return None;
        }

        if position.cmpge(IVec3::splat(CHUNK_SIZE)).any() {
            return None;
        }

        let removed = self.storage.get(position);

        self.storage.set(position, *AIR);

        Some(removed)
    }

    #[inline]
    pub fn get(&self, position: impl Into<IVec3>) -> Option<usize> {
        let position = position.into();

        if position.cmplt(IVec3::ZERO).any() {
            return None;
        }

        if position.cmpge(IVec3::splat(CHUNK_SIZE)).any() {
            return None;
        }

        Some(self.storage.get(position))
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = usize> {
        self.storage.iter()
    }

    #[inline]
    pub fn iter_with_position(&self) -> impl Iterator<Item = (IVec3, usize)> {
        self.storage.iter().enumerate().map(|(i, block)| {
            let x = (i & 0xF) as i32;
            let z = ((i >> 4) & 0xF) as i32;
            let y = (i >> 8) as i32;

            (IVec3::new(x, y, z), block)
        })
    }
}

#[derive(Component, Debug, Copy, Clone, Eq, PartialEq)]
pub struct ChunkPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl ChunkPos {
    #[inline]
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub fn from_world_pos(pos: impl Into<Vec3>) -> Self {
        let pos = pos.into().floor().as_ivec3() / CHUNK_SIZE;

        unsafe { transmute(pos) }
    }

    #[inline]
    pub fn into_world_pos(self) -> Vec3 {
        Vec3::new(
            (self.x * CHUNK_SIZE) as f32,
            (self.y * CHUNK_SIZE) as f32,
            (self.z * CHUNK_SIZE) as f32,
        )
    }

    #[inline(always)]
    pub fn bounding_volume(self) -> Aabb3d {
        let world_pos = self.into_world_pos();

        Aabb3d {
            min: world_pos.into(),
            max: (world_pos + CHUNK_SIZE as f32).into(),
        }
    }

    #[inline(always)]
    pub fn distance_squared(self, rhs: Self) -> i32 {
        (self - rhs).length_squared()
    }

    #[inline(always)]
    pub const fn length_squared(self) -> i32 {
        self.dot(self)
    }

    #[inline(always)]
    pub const fn dot(self, rhs: Self) -> i32 {
        (self.x * rhs.x) + (self.y * rhs.y) + (self.z * rhs.z)
    }
}

impl From<IVec3> for ChunkPos {
    fn from(value: IVec3) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

impl From<Vec3> for ChunkPos {
    #[inline(always)]
    fn from(value: Vec3) -> Self {
        Self {
            x: value.x.floor() as i32,
            y: value.y.floor() as i32,
            z: value.z.floor() as i32,
        }
    }
}

impl Hash for ChunkPos {
    #[inline(always)]
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_i32(self.x);
        state.write_i32(self.y);
        state.write_i32(self.z);
    }
}

impl Display for ChunkPos {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

impl core::ops::Add for ChunkPos {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Sub for ChunkPos {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}
