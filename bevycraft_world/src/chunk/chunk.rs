use {
    crate::prelude::*,
    bevy::{
        ecs::component::Component,
        math::{bounding::Aabb3d, IVec3, Vec3},
    },
    bevycraft_core::blocks::AIR,
    std::{
        fmt::{Debug, Display, Formatter, Result},
        hash::{Hash, Hasher},
        mem::transmute,
        ops::{Add, Div, Mul, Sub},
        sync::Arc,
    },
};

pub const CHUNK_SIZE: i32 = 16;

pub const CHUNK_LEN: usize = (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as usize;

#[derive(Component)]
pub struct Chunk {
    pub storage: Arc<ChunkStorage>,

    pub dirty: bool,
}

impl Chunk {
    #[inline]
    pub fn empty() -> Self {
        Self {
            storage: Arc::new(ChunkStorage::Empty),
            dirty: false,
        }
    }

    #[inline]
    pub fn uniform(block: usize) -> Self {
        Self {
            storage: Arc::new(ChunkStorage::Single(block)),
            dirty: false,
        }
    }

    #[inline]
    pub fn set(&mut self, position: impl Into<IVec3>, block: usize) {
        let position = position.into();

        if !check_bounds(position) {
            return;
        }

        Arc::make_mut(&mut self.storage).set(position, block.into());

        self.dirty = true;
    }

    #[inline]
    pub fn remove(&mut self, position: impl Into<IVec3>) -> Option<usize> {
        let position = position.into();

        if !check_bounds(position) {
            return None;
        }

        let removed = self.storage.get(position);

        Arc::make_mut(&mut self.storage).set(position, *AIR);

        self.dirty = true;

        Some(removed)
    }

    #[inline]
    pub fn get(&self, position: impl Into<IVec3>) -> Option<usize> {
        let position = position.into();

        if !check_bounds(position) {
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

#[inline(always)]
pub(crate) fn check_bounds(position: IVec3) -> bool {
    if position.cmplt(IVec3::ZERO).any() {
        return false;
    }

    if position.cmpge(IVec3::splat(CHUNK_SIZE)).any() {
        return false;
    }

    true
}

#[derive(Component, Copy, Clone, Eq, PartialEq)]
pub struct ChunkPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Into<IVec3> for ChunkPos {
    fn into(self) -> IVec3 {
        IVec3::new(self.x, self.y, self.z)
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
    pub const fn into_world_pos(self) -> Vec3 {
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

impl Display for ChunkPos {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

impl Debug for ChunkPos {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_tuple("ChunkPos")
            .field(&self.x)
            .field(&self.y)
            .field(&self.z)
            .finish()
    }
}

impl Hash for ChunkPos {
    #[inline(always)]
    fn hash<H: Hasher>(&self, state: &mut H) {
        const MASK: u64 = 0x1FFFFF;
        const Y_SHIFT: u64 = 21;
        const Z_SHIFT: u64 = 42;

        let hash = (self.x as u64 & MASK)
            | ((self.y as u64 & MASK) << Y_SHIFT)
            | ((self.z as u64 & MASK) << Z_SHIFT);

        state.write_u64(hash);
    }
}

impl Add for ChunkPos {
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

impl Add<IVec3> for ChunkPos {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: IVec3) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Add<i32> for ChunkPos {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: i32) -> Self::Output {
        Self {
            x: self.x + rhs,
            y: self.y + rhs,
            z: self.z + rhs,
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

impl Sub<IVec3> for ChunkPos {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: IVec3) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Sub<i32> for ChunkPos {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: i32) -> Self::Output {
        Self {
            x: self.x - rhs,
            y: self.y - rhs,
            z: self.z - rhs,
        }
    }
}

impl Div for ChunkPos {
    type Output = Self;

    #[inline(always)]
    fn div(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
            z: self.z / rhs.z,
        }
    }
}

impl Div<IVec3> for ChunkPos {
    type Output = Self;

    #[inline(always)]
    fn div(self, rhs: IVec3) -> Self::Output {
        Self {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
            z: self.z / rhs.z,
        }
    }
}

impl Div<i32> for ChunkPos {
    type Output = Self;

    #[inline(always)]
    fn div(self, rhs: i32) -> Self::Output {
        Self {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

impl Mul for ChunkPos {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}

impl Mul<IVec3> for ChunkPos {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: IVec3) -> Self::Output {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}

impl Mul<i32> for ChunkPos {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: i32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}
