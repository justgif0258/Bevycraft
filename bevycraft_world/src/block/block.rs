use std::fmt::{Display, Formatter};
use crate::prelude::BlockDefinition;
use bevy::math::bounding::Aabb3d;
use builder_pattern::Builder;

#[derive(Builder, Debug, Clone, PartialEq)]
pub struct Block {
    #[into]
    #[public]
    #[default(BlockDefinition::default())]
    definition  : BlockDefinition,

    #[into]
    #[public]
    #[default(Vec::new())]
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
}

impl Display for Block {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Block")
    }
}