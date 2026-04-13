use crate::prelude::BlockDefinition;
use bevy::math::bounding::Aabb3d;
use builder_pattern::Builder;

#[derive(Debug, PartialEq)]
pub struct BlockRef<'a> {
    pub(crate) definition   : &'a BlockDefinition,
    pub(crate) shapes       : &'a [Aabb3d],
}

impl<'a> BlockRef<'a> {
    #[inline]
    #[must_use]
    pub const fn definition(&self) -> &'a BlockDefinition {
        self.definition
    }

    #[inline]
    #[must_use]
    pub const fn shapes(&self) -> &'a [Aabb3d] {
        self.shapes
    }
}

#[derive(Builder, Debug, Clone, PartialEq)]
pub struct Block {
    definition  : BlockDefinition,
    shapes      : Vec<Aabb3d>,
}

impl Block {
    #[inline(always)]
    #[must_use]
    pub const fn definition(&self) -> &BlockDefinition {
        &self.definition
    }

    #[inline(always)]
    #[must_use]
    pub const fn shapes(&self) -> &[Aabb3d] {
        self.shapes.as_slice()
    }

    #[inline(always)]
    #[must_use]
    pub const fn as_ref(&self) -> BlockRef<'_> {
        BlockRef {
            definition: &self.definition,
            shapes: &self.shapes.as_slice(),
        }
    }
}