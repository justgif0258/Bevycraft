use bevy::platform::collections::HashMap;
use bevy::platform::hash::NoOpHash;
use bevy::prelude::*;
use crate::prelude::{Section, SectionPool};

pub struct Chunk {
    pub pos: IVec2,

    pub sections: HashMap<i32, Section, NoOpHash>,

    pub dirty: bool,
}

impl Chunk {
    #[inline(always)]
    pub const fn new(pos: IVec2) -> Self {
        Self {
            pos,
            sections: HashMap::with_hasher(NoOpHash),
            dirty: false,
        }
    }

    #[inline(always)]
    pub fn set_at(
        &mut self,
        gc: &mut SectionPool,
        pos: impl Into<IVec3>,
        global_idx: u32
    ) {
        let pos = pos.into();

        let normalized = pos.rem_euclid(Section::SECTION_SIZE.as_ivec3());

        let y_idx = pos.y.div_euclid(Section::SECTION_SIZE.y as i32);

        if let Some(section) = self.sections.get_mut(&y_idx) {
            section.set_at(normalized.as_uvec3(), global_idx);

            return;
        }

        let mut new_section = Section::from_allocated(gc.alloc_zeroed())
            .unwrap();

        new_section.set_at(normalized.as_uvec3(), global_idx);

        self.sections.insert(y_idx, new_section);
    }

    #[inline(always)]
    pub fn get_at(
        &self,
        pos: impl Into<IVec3>,
    ) -> Option<u32> {
        let pos = pos.into();

        let normalized = pos.rem_euclid(Section::SECTION_SIZE.as_ivec3());

        let y_idx = pos.y.div_euclid(Section::SECTION_SIZE.y as i32);

        if let Some(section) = self.sections.get(&y_idx) {
            return Some(
                section.get_at(normalized.as_uvec3())
            );
        }

        None
    }
}