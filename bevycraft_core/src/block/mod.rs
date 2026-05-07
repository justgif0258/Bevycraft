use std::{hash::Hash, mem::transmute, num::NonZeroU32};

pub mod block;
pub mod block_behaviour;
pub mod block_flags;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BlockType {
    Air,
    Id(NonZeroU32),
}

impl BlockType {
    #[inline(always)]
    pub const fn new(value: u32) -> Self {
        unsafe { transmute(value) }
    }

    #[inline(always)]
    pub const fn raw(self) -> u32 {
        unsafe { transmute(self) }
    }

    #[inline(always)]
    pub const fn is_air(self) -> bool {
        matches!(self, BlockType::Air)
    }
}

impl Default for BlockType {
    #[inline(always)]
    fn default() -> Self {
        Self::Air
    }
}

impl Hash for BlockType {
    #[inline(always)]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(self.raw() as u64);
    }
}
