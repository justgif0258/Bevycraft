use std::iter::{Repeat, Take};

use crate::prelude::{CHUNK_LEN, CHUNK_SIZE};
use bevy::math::IVec3;
use bevycraft_core::prelude::{BlockType, PatternContainer, PatternIter};

pub enum ChunkStorage {
    Empty,
    Single(BlockType),
    Pattern(PatternContainer<BlockType, CHUNK_LEN>),
}

impl ChunkStorage {
    pub fn single(block: BlockType) -> Self {
        Self::Single(block)
    }

    #[inline]
    pub fn empty_pattern() -> Self {
        Self::Pattern(PatternContainer::new(BlockType::Air))
    }

    #[inline]
    pub fn get(&self, position: IVec3) -> BlockType {
        match self {
            Self::Empty => BlockType::Air,
            Self::Single(b) => *b,
            Self::Pattern(p) => {
                let idx = linearize(position);

                p.get(idx).copied().unwrap_or(BlockType::Air)
            }
        }
    }

    #[inline]
    pub fn set(&mut self, position: IVec3, block: BlockType) {
        let idx = linearize(position);

        match self {
            Self::Empty => {
                let mut container = PatternContainer::new(BlockType::Air);

                container.set(idx, block);

                *self = Self::Pattern(container);
            }
            Self::Single(b) => {
                let mut container = PatternContainer::new(*b);

                container.set(idx, block);

                *self = Self::Pattern(container);
            }
            Self::Pattern(p) => p.set(idx, block),
        }
    }

    #[inline]
    pub fn fill(&mut self, block: BlockType) {
        *self = Self::Single(block);
    }

    pub fn clear(&mut self) {
        *self = Self::Empty;
    }

    #[inline]
    pub fn optimize(&mut self) -> bool {
        match self {
            Self::Pattern(p) => {
                if let Some(single) = p.as_single().copied() {
                    if single == BlockType::Air {
                        *self = Self::Empty;
                    } else {
                        *self = Self::Single(single);
                    }

                    return true;
                }

                p.try_compress()
            }
            _ => false, // Empty and Single are already considered as compressed
        }
    }

    #[inline]
    pub fn iter(&self) -> ChunkIter<'_> {
        match self {
            Self::Empty => ChunkIter::Uniform(std::iter::repeat(BlockType::Air).take(CHUNK_LEN)),
            Self::Single(b) => ChunkIter::Uniform(std::iter::repeat(*b).take(CHUNK_LEN)),
            Self::Pattern(p) => ChunkIter::Pattern(p.iter()),
        }
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        match self {
            Self::Empty => true,
            _ => false,
        }
    }

    #[inline]
    pub const fn is_uniform(&self) -> bool {
        match self {
            Self::Single(_) => true,
            _ => false,
        }
    }

    #[inline]
    pub const fn is_pattern(&self) -> bool {
        match self {
            Self::Pattern(_) => true,
            _ => false,
        }
    }
}

pub enum ChunkIter<'a> {
    Uniform(Take<Repeat<BlockType>>),
    Pattern(PatternIter<'a, BlockType, CHUNK_LEN>),
}

impl Iterator for ChunkIter<'_> {
    type Item = BlockType;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Uniform(i) => i.next(),
            Self::Pattern(i) => i.next().copied(),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Uniform(i) => i.size_hint(),
            Self::Pattern(i) => i.size_hint(),
        }
    }
}

#[inline(always)]
const fn linearize(position: IVec3) -> usize {
    (position.x + (position.z * CHUNK_SIZE) + (position.y * CHUNK_SIZE * CHUNK_SIZE)) as usize
}
