use std::fmt;
use std::fmt::Formatter;
use std::hash::{Hash, Hasher};
use std::mem::transmute;
use std::ops::Sub;
use bevy::math::bounding::Aabb3d;
use bevy::prelude::*;
use crossbeam::queue::SegQueue;
use crate::generator::chunk_generator::{ChunkGenerator, ChunkSource};
use crate::prelude::*;

const CHUNK_ARRAY_LENGTH: usize = (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as usize;

pub const CHUNK_SIZE: i32 = 16;

pub type BlockArray = Box<[BlockType; CHUNK_ARRAY_LENGTH]>;

static CHUNK_POOL: SegQueue<BlockArray> = SegQueue::new();

#[derive(Component, Debug, Clone, Eq, PartialEq)]
pub struct Chunk {
    blocks: Option<BlockArray>,

    pub dirty: bool,
}

impl Chunk {
    #[inline]
    pub fn new_empty() -> Self {
        let blocks = if let Some(mut pooled) = CHUNK_POOL.pop() {
            pooled.fill(BlockType::Air);
            pooled
        } else {
            Box::new([BlockType::Air; CHUNK_ARRAY_LENGTH])
        };

        Self {
            blocks: Some(blocks),
            dirty: false,
        }
    }

    #[inline]
    pub fn new_from_source(source: ChunkSource, position: ChunkPos, blocks: BlockRecord) -> Self {
        let mut chunk = Chunk {
            blocks: if let Some(mut pooled) = CHUNK_POOL.pop() {
                pooled.fill(BlockType::Air);
                Some(pooled)
            } else { Some(Box::new([BlockType::Air; CHUNK_ARRAY_LENGTH])) },
            dirty: false,
        };

        source.fill(position, &mut chunk, blocks.clone());
        source.carve(position, &mut chunk);
        source.place_features(position, &mut chunk, blocks);

        chunk
    }

    #[inline]
    pub fn set(&mut self, position: impl Into<IVec3>, block: BlockType) {
        let position = position.into();

        if position.cmplt(IVec3::ZERO).any() { return }

        if position.cmpge(IVec3::splat(CHUNK_SIZE)).any() { return }

        let blocks = self.blocks.as_mut().unwrap();

        blocks[linearize(position)] = block;
    }

    #[inline]
    pub fn remove(&mut self, position: impl Into<IVec3>) -> Option<BlockType> {
        let position = position.into();

        if position.cmplt(IVec3::ZERO).any() { return None }

        if position.cmpge(IVec3::splat(CHUNK_SIZE)).any() { return None }

        let blocks = self.blocks.as_mut().unwrap();

        let linearized = linearize(position);

        let paletted = blocks[linearized];

        blocks[linearized] = BlockType::Air;
        
        Some(paletted)
    }

    #[inline]
    pub fn get(&self, position: impl Into<IVec3>) -> Option<BlockType> {
        let position = position.into();

        if position.cmplt(IVec3::ZERO).any() { return None }

        if position.cmpge(IVec3::splat(CHUNK_SIZE)).any() { return None }

        let blocks = self.blocks.as_ref().unwrap();

        Some(blocks[linearize(position)])
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = BlockType> {
        self.blocks.as_ref().unwrap()
            .iter()
            .copied()
    }

    #[inline]
    pub fn iter_with_position(&self) -> impl Iterator<Item = (IVec3, BlockType)> {
        let blocks = self.blocks.as_ref().unwrap();

        blocks.iter()
            .enumerate()
            .map(|(i, &block)| {
                let x = (i & 0xF) as i32;
                let z = ((i >> 4) & 0xF) as i32;
                let y = (i >> 8) as i32;

                (IVec3::new(x, y, z), block)
            })
    }

    #[inline]
    pub fn get_pool() -> &'static SegQueue<BlockArray> {
        &CHUNK_POOL
    }
}

impl Drop for Chunk {
    #[inline]
    fn drop(&mut self) {
        CHUNK_POOL.push(self.blocks.take().unwrap());
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

impl fmt::Display for ChunkPos {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {}, {}]", self.x, self.y, self.z)
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

#[inline(always)]
const fn linearize(position: IVec3) -> usize {
    (position.x + (position.z * CHUNK_SIZE) + (position.y * CHUNK_SIZE * CHUNK_SIZE)) as usize
}