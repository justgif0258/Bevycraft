use crate::prelude::*;
use bevy::math::bounding::Aabb3d;
use bevycraft_macros::Registrar;
use builder_pattern::Builder;

#[derive(Registrar, Builder, Debug, PartialEq)]
#[registrar(default = "air")]
pub struct Block {
    #[into]
    #[public]
    #[default(BlockBehaviour::default())]
    behaviour: BlockBehaviour,

    #[into]
    #[public]
    #[default(Box::new([]))]
    shapes: Box<[Aabb3d]>,
}

impl Default for Block {
    #[inline(always)]
    fn default() -> Self {
        Self {
            behaviour: BlockBehaviour::default(),
            shapes: Box::new([]),
        }
    }
}

impl Block {
    #[must_use]
    #[inline(always)]
    pub const fn air(&self) -> bool {
        self.behaviour.flags.contains(BlockFlags::AIR)
    }

    #[inline(always)]
    pub const fn hardness(&self) -> f32 {
        self.behaviour.hardness
    }

    #[inline(always)]
    pub const fn toughness(&self) -> f32 {
        self.behaviour.toughness
    }

    #[inline(always)]
    pub const fn friction(&self) -> f32 {
        self.behaviour.friction
    }

    #[inline(always)]
    pub const fn viscosity(&self) -> f32 {
        self.behaviour.bounciness
    }

    #[inline(always)]
    pub const fn collidable(&self) -> bool {
        self.behaviour.flags.contains(BlockFlags::COLLIDABLE)
    }

    #[inline(always)]
    pub const fn occludable(&self) -> bool {
        self.behaviour.flags.contains(BlockFlags::OCCLUDABLE)
    }

    #[inline(always)]
    pub const fn see_through(&self) -> bool {
        self.behaviour.flags.contains(BlockFlags::SEE_THROUGH)
    }

    #[inline(always)]
    pub const fn replaceable(&self) -> bool {
        self.behaviour.flags.contains(BlockFlags::REPLACEABLE)
    }

    #[inline(always)]
    pub const fn can_support(&self) -> bool {
        self.behaviour.flags.contains(BlockFlags::CAN_SUPPORT)
    }

    #[inline(always)]
    pub const fn does_connect(&self) -> bool {
        self.behaviour.flags.contains(BlockFlags::DOES_CONNECT)
    }

    #[inline(always)]
    pub const fn does_spawn(&self) -> bool {
        self.behaviour.flags.contains(BlockFlags::DOES_SPAWN)
    }

    #[inline(always)]
    pub const fn climbable(&self) -> bool {
        self.behaviour.flags.contains(BlockFlags::CLIMBABLE)
    }

    #[inline(always)]
    pub const fn passable(&self) -> bool {
        self.behaviour.flags.contains(BlockFlags::PASSABLE)
    }

    #[inline(always)]
    pub const fn shapes(&self) -> &[Aabb3d] {
        &self.shapes
    }
}
