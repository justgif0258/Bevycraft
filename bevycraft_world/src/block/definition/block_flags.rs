bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
    pub struct BlockFlags: u16 {
        const AIR           = 1 << u16::BITS - 1;
        const COLLIDABLE    = 1 << 0;
        const OCCLUDABLE    = 1 << 1;
        const GREEDY_MESH   = 1 << 2;
        const CUTOUT        = 1 << 3;
        const TRANSLUCENT   = 1 << 4;
        const REPLACEABLE   = 1 << 5;
        const CAN_SUPPORT   = 1 << 6;
        const DOES_CONNECT  = 1 << 7;
        const DOES_SPAWN    = 1 << 8;
        const CLIMBABLE     = 1 << 9;
        const PASSABLE      = 1 << 10;
    }
}

impl Default for BlockFlags {
    #[inline(always)]
    fn default() -> Self {
        Self::AIR
    }
}