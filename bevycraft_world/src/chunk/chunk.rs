use std::hash::{Hash, Hasher};
use std::mem::transmute_copy;
use std::sync::Arc;
use bevy::platform::collections::HashMap;
use bevy::platform::hash::NoOpHash;
use bevy::prelude::*;
use crate::prelude::*;

#[derive(Component, Debug, Copy, Clone, Eq, PartialEq)]
pub struct ChunkPos(pub IVec2);

impl Hash for ChunkPos {
    #[inline(always)]
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(unsafe { transmute_copy(self) })
    }
}

impl ChunkPos {
    #[inline]
    pub const fn new(x: i32, z: i32) -> Self {
        Self(IVec2::new(x, z))
    }
    
    #[inline]
    pub fn from_world_pos(pos: impl Into<Vec3>) -> Self {
        let world_pos = pos.into();
        
        Self(IVec2::new(
            (world_pos.x / SECTION_SIZE as f32).floor() as i32,
            (world_pos.z / SECTION_SIZE as f32).floor() as i32,
        ))
    }
    
    #[inline(always)]
    pub const fn into_world_pos(self) -> Vec3 {
        Vec3::new((self.0.x * SECTION_SIZE) as f32, 0.0, (self.0.y * SECTION_SIZE) as f32)
    }

    #[inline(always)]
    pub fn distance_squared(&self, other: ChunkPos) -> i32 {
        self.0.distance_squared(other.0)
    }

    #[inline(always)]
    pub fn neighbors(&self) -> [ChunkPos; 4] {
        [
            Self(self.0 + IVec2::X),
            Self(self.0 - IVec2::X),
            Self(self.0 + IVec2::Y),
            Self(self.0 - IVec2::Y),
        ]
    }
}

#[derive(Component, Default)]
pub struct Chunk {
    pub sections: HashMap<SectionIndex, PalettedSection, NoOpHash>,
}

impl Chunk {
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            sections: HashMap::with_hasher(NoOpHash),
        }
    }

    #[inline(always)]
    pub fn generate_using(
        position: ChunkPos,
        blocks: Arc<BlockRecord>,
        generator: ActiveWorldGenerator,
    ) -> Self {
        let mut chunk = Self::new();

        let position = position.into();

        generator.generate_base_terrain(
            position,
            &mut chunk,
            blocks.clone(),
        );

        generator.carve_terrain(
            position,
            &mut chunk,
        );

        generator.generate_features(
            position,
            &mut chunk,
            blocks,
        );

        chunk
    }

    #[inline(always)]
    pub fn set_at(
        &mut self,
        position: impl Into<IVec3>,
        block_type: BlockType
    ) {
        let position = position.into();

        if position.cmplt(IVec3::ZERO).any() {
            return;
        }

        if position.x >= SECTION_SIZE || position.z >= SECTION_SIZE {
            return;
        }

        let normalized = position.rem_euclid(IVec3::splat(SECTION_SIZE));

        let y_idx = SectionIndex::from_world_height(position.y);

        if let Some(section) = self.sections.get_mut(&y_idx) {
            section.set(normalized, block_type);

            return;
        }

        let mut new_section = PalettedSection::new();

        new_section.set(normalized, block_type);

        self.sections.insert(y_idx, new_section);
    }

    #[inline(always)]
    pub fn remove_at(&mut self, position: impl Into<IVec3>) -> BlockType {
        let position = position.into();

        if position.cmplt(IVec3::ZERO).any() {
            return BlockType::Air;
        }

        if position.x >= SECTION_SIZE || position.z >= SECTION_SIZE {
            return BlockType::Air;
        }

        let y_idx = SectionIndex::from_world_height(position.y);

        if let Some(section) = self.sections.get_mut(&y_idx) {
            let normalized = position.rem_euclid(IVec3::splat(SECTION_SIZE));

            return section.remove(normalized);
        }
        
        BlockType::Air
    }

    #[inline(always)]
    pub fn get_at(
        &self,
        position: impl Into<IVec3>,
    ) -> BlockType {
        let position = position.into();

        if position.cmplt(IVec3::ZERO).any() {
            return BlockType::Air;
        }

        if position.x >= SECTION_SIZE || position.z >= SECTION_SIZE {
            return BlockType::Air;
        }

        let y_idx = SectionIndex::from_world_height(position.y);

        if let Some(section) = self.sections.get(&y_idx) {
            let normalized = IVec3::new(
                position.x,
                position.y.rem_euclid(SECTION_SIZE),
                position.z,
            );

            return section.get(normalized);
        }

        BlockType::Air
    }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct SectionIndex(pub i32);

impl SectionIndex {
    #[inline(always)]
    pub const fn from_world_height(height: i32) -> Self {
        Self(height.div_euclid(SECTION_SIZE))
    }
    
    #[inline(always)]
    pub const fn into_world_height(self) -> i32 {
        self.0 * SECTION_SIZE
    }
}

impl Hash for SectionIndex {
    #[inline(always)]
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.0 as u64)
    }
}