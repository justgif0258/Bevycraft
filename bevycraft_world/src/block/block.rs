use bevycraft_core::prelude::Recordable;
use builder_pattern::Builder;

pub trait Block: Recordable {
    fn get_behaviour(&self) -> BlockBehaviour;
}

#[derive(Builder, Debug)]
pub struct BlockBehaviour {
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
    emission: f32,

    #[into]
    #[public]
    #[default(BlockFlags::empty())]
    flags: BlockFlags,
}

impl BlockBehaviour {
    #[inline]
    pub const fn hardness(&self) -> f32 {
        self.hardness
    }

    #[inline]
    pub const fn toughness(&self) -> f32 {
        self.toughness
    }

    #[inline]
    pub const fn friction(&self) -> f32 {
        self.friction
    }

    #[inline]
    pub const fn emission(&self) -> f32 {
        self.emission
    }

    #[inline]
    pub const fn air(&self) -> bool {
        self.flags.contains(BlockFlags::AIR)
    }

    #[inline]
    pub const fn collidable(&self) -> bool {
        self.flags.contains(BlockFlags::COLLIDABLE)
    }

    #[inline]
    pub const fn occludable(&self) -> bool {
        self.flags.contains(BlockFlags::OCCLUDABLE)
    }

    #[inline]
    pub const fn translucent(&self) -> bool {
        self.flags.contains(BlockFlags::TRANSLUCENT)
    }

    #[inline]
    pub const fn replaceable(&self) -> bool {
        self.flags.contains(BlockFlags::REPLACEABLE)
    }

    #[inline]
    pub const fn can_support(&self) -> bool {
        self.flags.contains(BlockFlags::CAN_SUPPORT)
    }

    #[inline]
    pub const fn does_spawn(&self) -> bool {
        self.flags.contains(BlockFlags::DOES_SPAWN)
    }

    #[inline]
    pub const fn does_connect(&self) -> bool {
        self.flags.contains(BlockFlags::DOES_CONNECT)
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct BlockFlags: u8 {
        const AIR = 0b1000_0000;
        const COLLIDABLE = 0b0000_0001;
        const OCCLUDABLE = 0b0000_0010;
        const TRANSLUCENT = 0b0000_0100;
        const REPLACEABLE = 0b0000_1000;
        const CAN_SUPPORT = 0b0001_0000;
        const DOES_SPAWN = 0b0010_0000;
        const DOES_CONNECT = 0b0100_0000;
    }
}
