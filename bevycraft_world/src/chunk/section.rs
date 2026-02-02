use bevy::prelude::*;
use bevycraft_core::prelude::PackedArrayU32;

pub struct Section<T: PartialEq + Eq> {
    states: PackedArrayU32,
    content: Vec<T>,
    y: usize,
}

impl<T: PartialEq + Eq> Section<T> {
    const SECTION_LEN: usize = 4096;

    const SECTION_SIZE: UVec3 = UVec3::new(16, 16, 16);

    pub fn new(y: usize) -> Self {
        Self {
            states: PackedArrayU32::zeroed(Self::SECTION_LEN),
            content: Vec::new(),
            y,
        }
    }

    #[inline]
    pub fn set(&mut self, pos: UVec3, state: T) {
        if self.states.is_empty() {
            self.states.allocate()
                .expect("Failed to allocate array");
        }

        let idx = self.content.len();
        self.content.push(state);

        self.states.set(Self::map_to_flat_index(pos), idx as u32)
    }

    #[inline]
    pub fn get(&self, pos: UVec3) -> Option<&T> {
        if self.states.is_empty() {
            return None;
        }

        let idx = self.states.get(Self::map_to_flat_index(pos));

        self.content.get(idx as usize)
    }

    const fn map_to_flat_index(pos: UVec3) -> usize {
        (pos.x + (pos.z * Self::SECTION_SIZE.x) + (pos.y * Self::SECTION_SIZE.x * Self::SECTION_SIZE.z)) as usize
    }
}