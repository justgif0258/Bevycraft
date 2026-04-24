use std::fmt;
use std::fmt::Formatter;
use std::hash::{Hash, Hasher};
use std::mem::transmute;
use std::ops::{Sub, SubAssign};
use bevy::math::bounding::Aabb3d;
use bevy::platform::collections::HashMap;
use bevy::platform::hash::NoOpHash;
use bevy::prelude::*;
use crossbeam::queue::SegQueue;
use crate::generator::chunk_generator::{ChunkGenerator, ChunkSource};
use crate::prelude::*;

const CHUNK_ARRAY_LENGTH: usize = (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as usize;

pub const CHUNK_SIZE: i32 = 16;

pub type BlockArray = Box<[u32; CHUNK_ARRAY_LENGTH]>;

static CHUNK_POOL: SegQueue<BlockArray> = SegQueue::new();

#[derive(Component, Debug, Clone, Eq, PartialEq)]
pub struct Chunk {
    blocks: Option<BlockArray>,
    palette: BlockPalette,

    pub dirty: bool,
}

impl Chunk {
    #[inline]
    pub fn new_empty() -> Self {
        let blocks = if let Some(mut pooled) = CHUNK_POOL.pop() {
            pooled.fill(0u32);
            pooled
        } else {
            Box::new([0u32; CHUNK_ARRAY_LENGTH])
        };

        Self {
            blocks: Some(blocks),
            palette: BlockPalette::new(),
            dirty: false,
        }
    }

    #[inline]
    pub fn new_from_source(source: ChunkSource, position: ChunkPos, blocks: BlockRecord) -> Self {
        let mut chunk = Chunk {
            blocks: if let Some(pooled) = CHUNK_POOL.pop() {
                Some(pooled)
            } else { Some(Box::new([0u32; CHUNK_ARRAY_LENGTH])) },
            palette: BlockPalette::new(),
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

        let paletted = self.palette.get_or_create_entry(block);

        blocks[linearize(position)] = paletted;
    }

    #[inline]
    pub fn remove(&mut self, position: impl Into<IVec3>) -> Option<BlockType> {
        let position = position.into();

        if position.cmplt(IVec3::ZERO).any() { return None }

        if position.cmpge(IVec3::splat(CHUNK_SIZE)).any() { return None }

        let blocks = self.blocks.as_mut().unwrap();

        let linearized = linearize(position);

        let paletted = blocks[linearized];

        blocks[linearized] = 0;

        self.palette.retrieve_and_dec_entry(paletted)
    }

    #[inline]
    pub fn get(&self, position: impl Into<IVec3>) -> Option<BlockType> {
        let position = position.into();

        if position.cmplt(IVec3::ZERO).any() { return None }

        if position.cmpge(IVec3::splat(CHUNK_SIZE)).any() { return None }

        let blocks = self.blocks.as_ref().unwrap();

        let paletted = blocks[linearize(position)];

        self.palette.entries.get(paletted as usize).copied()
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = BlockType> {
        self.blocks.as_ref().unwrap()
            .iter()
            .map(|&index| {
                self.palette.entries[index as usize]
            })
    }

    #[inline]
    pub fn iter_with_position(&self) -> impl Iterator<Item = (IVec3, BlockType)> {
        let blocks = self.blocks.as_ref().unwrap();

        blocks.iter()
            .enumerate()
            .map(|(i, &index)| {
                let x = (i & 0xF) as i32;
                let y = ((i >> 4) & 0xF) as i32;
                let z = (i >> 8) as i32;

                (IVec3::new(x, y, z), self.palette.entries[index as usize])
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
        write!(f, "Chunk position = [{}, {}, {}]", self.x, self.y, self.z)
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BlockPalette {
    entries: Vec<BlockType>,
    mappings: HashMap<BlockType, (u32, u32), NoOpHash>,
    free_list: Vec<u32>,
}

impl BlockPalette {
    #[inline]
    pub fn new() -> Self {
        let mut entries = Vec::with_capacity(8);

        entries.push(BlockType::Air);

        Self {
            entries,
            mappings: HashMap::with_capacity_and_hasher(8, NoOpHash),
            free_list: Vec::with_capacity(8),
        }
    }

    #[inline(always)]
    pub fn get_or_create_entry(&mut self, entry: BlockType) -> u32 {
        if entry.is_air() { return 0u32 }

        if let Some(index) = self.mappings.get_mut(&entry) {
            index.1 += 1;

            return index.0;
        }

        if let Some(freed) = self.free_list.pop() {
            self.entries[freed as usize] = entry;
            self.mappings.insert(entry, (freed, 1));

            return freed;
        }

        let next_index = self.entries.len() as u32;

        self.entries.push(entry);
        self.mappings.insert(entry, (next_index, 1));

        next_index
    }

    #[inline(always)]
    pub fn retrieve_and_dec_entry(&mut self, paletted: u32) -> Option<BlockType> {
        if let Some(&block) = self.entries.get(paletted as usize) {
            self.decrement_ref_count(block);

            return Some(block);
        }

        None
    }

    #[inline(always)]
    pub fn decrement_ref_count(&mut self, entry: BlockType) {
        let mut i = 0u32;
        let mut c = 0u32;

        if let Some((index, counter)) = self.mappings.get_mut(&entry) {
            counter.sub_assign(1);

            i = *index;
            c = *counter;
        } else { return }

        if c == 0 {
            self.mappings.remove(&entry);
            self.free_list.push(i);
        }
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.mappings.is_empty()
            && self.entries.len() <= 1
    }

    #[inline(always)]
    pub const fn needs_resize(&self) -> bool {
        self.free_list.len() > 0
    }
}

#[inline(always)]
const fn linearize(position: IVec3) -> usize {
    (position.x + (position.z * CHUNK_SIZE) + (position.y * CHUNK_SIZE * CHUNK_SIZE)) as usize
}