use builder_pattern::Builder;
use crate::prelude::{BlockFlags};

#[derive(Builder, Debug, Clone, PartialEq)]
pub struct BlockDefinition {
    #[into]
    #[public]
    #[default(0.0)]
    hardness: f32,

    #[into]
    #[public]
    #[default(0.0)]
    toughness: f32,

    #[into]
    #[public]
    #[default(0.6)]
    friction: f32,

    #[into]
    #[public]
    #[default(0.0)]
    viscosity: f32,

    #[into]
    #[public]
    #[default(BlockFlags::empty())]
    flags: BlockFlags,
}

impl Default for BlockDefinition {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
            .friction(0.0)
            .flags(BlockFlags::AIR)
            .build()
    }
}

impl BlockDefinition {
    #[inline(always)]
    pub const fn hardness(&self) -> f32 {
        self.hardness
    }

    #[inline(always)]
    pub const fn toughness(&self) -> f32 {
        self.toughness
    }

    #[inline(always)]
    pub const fn friction(&self) -> f32 {
        self.friction
    }

    #[inline(always)]
    pub const fn viscosity(&self) -> f32 {
        self.viscosity
    }

    #[inline(always)]
    pub const fn air(&self) -> bool {
        self.flags.contains(BlockFlags::AIR)
    }

    #[inline(always)]
    pub const fn collidable(&self) -> bool {
        self.flags.contains(BlockFlags::COLLIDABLE)
    }

    #[inline(always)]
    pub const fn occludable(&self) -> bool {
        self.flags.contains(BlockFlags::OCCLUDABLE)
    }

    #[inline(always)]
    pub const fn greedy_meshable(&self) -> bool {
        self.flags.contains(BlockFlags::GREEDY_MESH)
    }

    #[inline(always)]
    pub const fn translucent(&self) -> bool {
        self.flags.contains(BlockFlags::TRANSLUCENT)
    }

    #[inline(always)]
    pub const fn replaceable(&self) -> bool {
        self.flags.contains(BlockFlags::REPLACEABLE)
    }

    #[inline(always)]
    pub const fn can_support(&self) -> bool {
        self.flags.contains(BlockFlags::CAN_SUPPORT)
    }

    #[inline(always)]
    pub const fn does_connect(&self) -> bool {
        self.flags.contains(BlockFlags::DOES_CONNECT)
    }

    #[inline(always)]
    pub const fn does_spawn(&self) -> bool {
        self.flags.contains(BlockFlags::DOES_SPAWN)
    }

    #[inline(always)]
    pub const fn climbable(&self) -> bool {
        self.flags.contains(BlockFlags::CLIMBABLE)
    }

    #[inline(always)]
    pub const fn passable(&self) -> bool {
        self.flags.contains(BlockFlags::PASSABLE)
    }
}
