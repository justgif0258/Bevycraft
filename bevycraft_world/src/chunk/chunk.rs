use std::sync::Arc;
use bevy::platform::collections::HashMap;
use bevy::platform::hash::NoOpHash;
use bevy::prelude::*;
use crate::prelude::*;

#[derive(Component, Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct ChunkPos(pub IVec2);

impl ChunkPos {
    #[inline]
    pub const fn new(x: i32, z: i32) -> Self {
        Self(IVec2::new(x, z))
    }
    
    #[inline]
    pub const fn from_world_pos(x: i32, z: i32) -> Self {
        Self(IVec2::new(x.div_euclid(SECTION_SIZE), z.div_euclid(SECTION_SIZE)))
    }
    
    #[inline]
    pub fn from_3d_world_pos(pos: Vec3) -> Self {
        Self(IVec2::new(pos.x.div_euclid(SECTION_SIZE as f32).floor() as i32, pos.z.div_euclid(SECTION_SIZE as f32).floor() as i32))
    }
    
    #[inline]
    pub const fn as_world_pos(&self) -> IVec2 {
        IVec2::new(self.0.x * SECTION_SIZE, self.0.y * SECTION_SIZE)
    }
    
    #[inline(always)]
    pub fn neighbors(&self) -> [IVec2; 4] {
        [
            self.0 + IVec2::X, self.0 - IVec2::X,
            self.0 + IVec2::Y, self.0 - IVec2::Y,
        ]
    }
}

#[derive(Component, Default)]
pub struct Chunk {
    pub sections: HashMap<u64, PalettedSection, NoOpHash>,

    pub dirty: bool,
}

impl Chunk {
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            sections: HashMap::with_hasher(NoOpHash),
            dirty: false,
        }
    }

    #[inline(always)]
    pub fn generate_using(
        position: impl Into<IVec2>,
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

        let y_idx = position.y.div_euclid(SECTION_SIZE) as u64;

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

        let y_idx = position.y.div_euclid(SECTION_SIZE) as u64;

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

        let y_idx = position.y.div_euclid(SECTION_SIZE) as u64;

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