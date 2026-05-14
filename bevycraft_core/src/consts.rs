use {
    crate::block::flags::BlockFlags,
    bevy::math::{bounding::Aabb3d, Vec3A},
    std::sync::LazyLock,
};

pub const FULL_SHAPE: [Aabb3d; 1] = [Aabb3d {
    min: Vec3A::new(0.0, 0.0, 0.0),
    max: Vec3A::new(1.0, 1.0, 1.0),
}];

pub const SLAB_SHAPE: [Aabb3d; 1] = [Aabb3d {
    min: Vec3A::new(0.0, 0.0, 0.0),
    max: Vec3A::new(1.0, 0.5, 1.0),
}];

pub const STAIR_SHAPE: [Aabb3d; 2] = [
    Aabb3d {
        min: Vec3A::new(0.0, 0.0, 0.0),
        max: Vec3A::new(1.0, 0.5, 1.0),
    },
    Aabb3d {
        min: Vec3A::new(0.0, 0.5, 0.0),
        max: Vec3A::new(1.0, 1.0, 0.5),
    },
];

pub const TRAPDOOR_SHAPE: [Aabb3d; 1] = [Aabb3d {
    min: Vec3A::new(0.0, 0.0, 0.0),
    max: Vec3A::new(1.0, 0.25, 1.0),
}];

pub const FULL_BLOCK: LazyLock<BlockFlags> = LazyLock::new(|| {
    BlockFlags::OCCLUDABLE
        | BlockFlags::COLLIDABLE
        | BlockFlags::DOES_SPAWN
        | BlockFlags::CAN_SUPPORT
});
