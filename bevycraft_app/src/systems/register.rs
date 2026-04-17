use bevy::math::bounding::Aabb3d;
use bevy::prelude::*;
use bevycraft_core::prelude::*;
use bevycraft_world::prelude::*;

const FULL_SHAPE: Aabb3d = Aabb3d { min: Vec3A::new(0.0, 0.0, 0.0), max: Vec3A::new(1.0, 1.0, 1.0), };

const HALF_SHAPE: Aabb3d = Aabb3d { min: Vec3A::new(0.0, 0.0, 0.0), max: Vec3A::new(1.0, 0.5, 1.0), };


pub fn register_blocks() -> BlockRecord {
    let mut commit = BlockCommit::new();
    
    commit.push(
        AssetLocation::with_default_namespace("air"),
        Block::new()
            .definition(
                BlockDefinition::new()
                    .friction(0.0)
                    .flags(BlockFlags::AIR)
                    .build()
            )
            .shapes(vec![])
            .build()
    );

    commit.push(
        AssetLocation::with_default_namespace("grass_block"),
        Block::new()
            .definition(
                BlockDefinition::new()
                    .hardness(0.65)
                    .toughness(0.65)
                    .flags(BlockFlags::COLLIDABLE
                        | BlockFlags::OCCLUDABLE
                        | BlockFlags::CAN_SUPPORT
                        | BlockFlags::DOES_SPAWN
                    )
                    .build()
            )
            .shapes(vec![FULL_SHAPE])
            .build()
    );

    commit.push(
        AssetLocation::with_default_namespace("dirt"),
        Block::new()
            .definition(
                BlockDefinition::new()
                    .hardness(0.5)
                    .toughness(0.5)
                    .flags(BlockFlags::COLLIDABLE
                        | BlockFlags::OCCLUDABLE
                        | BlockFlags::CAN_SUPPORT
                        | BlockFlags::DOES_SPAWN
                    )
                    .build()
            )
            .shapes(vec![FULL_SHAPE])
            .build()
    );

    commit.push(
        AssetLocation::with_default_namespace("stone"),
        Block::new()
            .definition(
                BlockDefinition::new()
                    .hardness(2.0)
                    .toughness(6.0)
                    .flags(BlockFlags::COLLIDABLE
                        | BlockFlags::OCCLUDABLE
                        | BlockFlags::CAN_SUPPORT
                        | BlockFlags::DOES_SPAWN
                    )
                    .build()
            )
            .shapes(vec![FULL_SHAPE])
            .build()
    );

    commit.push(
        AssetLocation::with_default_namespace("bedrock"),
        Block::new()
            .definition(
                BlockDefinition::new()
                    .hardness(f32::INFINITY)
                    .toughness(f32::INFINITY)
                    .flags(BlockFlags::COLLIDABLE
                        | BlockFlags::OCCLUDABLE
                        | BlockFlags::CAN_SUPPORT
                    )
                    .build()
            )
            .shapes(vec![FULL_SHAPE])
            .build()
    );

    commit.push(
        AssetLocation::with_default_namespace("oak_log"),
        Block::new()
            .definition(
                BlockDefinition::new()
                    .hardness(2.0)
                    .toughness(2.0)
                    .flags(BlockFlags::COLLIDABLE
                        | BlockFlags::OCCLUDABLE
                        | BlockFlags::CAN_SUPPORT
                        | BlockFlags::DOES_SPAWN
                    )
                    .build()
            )
            .shapes(vec![FULL_SHAPE])
            .build()
    );

    commit.push(
        AssetLocation::with_default_namespace("oak_planks"),
        Block::new()
            .definition(
                BlockDefinition::new()
                    .hardness(2.0)
                    .toughness(3.0)
                    .flags(BlockFlags::COLLIDABLE
                        | BlockFlags::OCCLUDABLE
                        | BlockFlags::CAN_SUPPORT
                        | BlockFlags::DOES_SPAWN
                    )
                    .build()
            )
            .shapes(vec![FULL_SHAPE])
            .build()
    );

    commit.push(
        AssetLocation::with_default_namespace("oak_planks_slab"),
        Block::new()
            .definition(
                BlockDefinition::new()
                    .hardness(2.0)
                    .toughness(3.0)
                    .flags(BlockFlags::COLLIDABLE
                        | BlockFlags::OCCLUDABLE
                        | BlockFlags::CAN_SUPPORT
                        | BlockFlags::DOES_SPAWN
                    )
                    .build()
            )
            .shapes(vec![HALF_SHAPE])
            .build()
    );

    commit.push(
        AssetLocation::with_default_namespace("oak_planks_stair"),
        Block::new()
            .definition(
                BlockDefinition::new()
                    .hardness(2.0)
                    .toughness(3.0)
                    .flags(BlockFlags::COLLIDABLE
                        | BlockFlags::OCCLUDABLE
                        | BlockFlags::CAN_SUPPORT
                        | BlockFlags::DOES_SPAWN
                    )
                    .build()
            )
            .shapes(vec![HALF_SHAPE, Aabb3d::from_min_max([0.0, 4.0, 0.0], [8.0, 8.0, 4.0])])
            .build()
    );

    commit.push(
        AssetLocation::with_default_namespace("oak_leaves"),
        Block::new()
            .definition(
                BlockDefinition::new()
                    .hardness(0.2)
                    .toughness(0.2)
                    .flags(BlockFlags::COLLIDABLE
                        | BlockFlags::OCCLUDABLE
                        | BlockFlags::CAN_SUPPORT
                        | BlockFlags::DOES_SPAWN
                    )
                    .build()
            )
            .shapes(vec![FULL_SHAPE])
            .build()
    );

    commit.push(
        AssetLocation::with_default_namespace("snow_block"),
        Block::new()
            .definition(
                BlockDefinition::new()
                    .hardness(0.2)
                    .toughness(0.2)
                    .flags(BlockFlags::COLLIDABLE
                        | BlockFlags::OCCLUDABLE
                        | BlockFlags::CAN_SUPPORT
                        | BlockFlags::DOES_SPAWN
                    )
                    .build()
            )
            .shapes(vec![FULL_SHAPE])
            .build()
    );

    commit.push(
        AssetLocation::with_default_namespace("sand"),
        Block::new()
            .definition(
                BlockDefinition::new()
                    .hardness(0.5)
                    .toughness(0.5)
                    .flags(BlockFlags::COLLIDABLE
                        | BlockFlags::OCCLUDABLE
                        | BlockFlags::CAN_SUPPORT
                        | BlockFlags::DOES_SPAWN
                    )
                    .build()
            )
            .shapes(vec![FULL_SHAPE])
            .build()
    );

    BlockRecord::finish(commit)
}