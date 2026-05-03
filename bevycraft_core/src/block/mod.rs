use std::{
    hash::Hash,
    mem::{transmute, transmute_copy},
    num::NonZeroU32,
};

pub mod block;
pub mod block_behaviour;
pub mod block_flags;
pub mod blocks;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    #[inline]
    fn default() -> Self {
        Self::Air
    }
}

impl Hash for BlockType {
    #[inline(always)]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe {
            state.write_u64(transmute_copy::<_, u32>(self) as u64);
        }
    }
}

impl From<BlockType> for usize {
    #[inline(always)]
    fn from(value: BlockType) -> Self {
        unsafe { transmute::<_, u32>(value) as usize }
    }
}

impl From<usize> for BlockType {
    #[inline(always)]
    fn from(value: usize) -> Self {
        debug_assert!(value <= u32::MAX as usize);

        unsafe { transmute(value as u32) }
    }
}
