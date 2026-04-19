use std::sync::Arc;
use bevy::platform::collections::HashMap;
use bevy::platform::hash::NoOpHash;
use bevy::prelude::*;
use crate::prelude::*;

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
        global_index: u32
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
            section.set(normalized, global_index);

            return;
        }

        let mut new_section = PalettedSection::new();

        new_section.set(normalized, global_index);

        self.sections.insert(y_idx, new_section);
    }

    #[inline(always)]
    pub fn remove_at(&mut self, position: impl Into<IVec3>) {
        let position = position.into();

        if position.cmplt(IVec3::ZERO).any() {
            return;
        }

        if position.x >= SECTION_SIZE || position.z >= SECTION_SIZE {
            return;
        }

        let y_idx = position.y.div_euclid(SECTION_SIZE) as u64;

        if let Some(section) = self.sections.get_mut(&y_idx) {
            let normalized = position.rem_euclid(IVec3::splat(SECTION_SIZE));

            section.remove(normalized);
        }
    }

    #[inline(always)]
    pub fn get_at(
        &self,
        position: impl Into<IVec3>,
    ) -> Option<u32> {
        let position = position.into();

        if position.cmplt(IVec3::ZERO).any() {
            return None;
        }

        if position.x >= SECTION_SIZE || position.z >= SECTION_SIZE {
            return None;
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

        None
    }
}