use std::sync::LazyLock;
use bevy::math::bounding::Aabb3d;
use bevy::prelude::*;
use bevycraft_core::prelude::*;
use bevycraft_world::prelude::*;

const FULL_SHAPE: Aabb3d = Aabb3d { min: Vec3A::new(0.0, 0.0, 0.0), max: Vec3A::new(1.0, 1.0, 1.0), };

const HALF_SHAPE: Aabb3d = Aabb3d { min: Vec3A::new(0.0, 0.0, 0.0), max: Vec3A::new(1.0, 0.5, 1.0), };

const FULL_BLOCK: LazyLock<BlockFlags> =  LazyLock::new(||
    BlockFlags::OCCLUDABLE
        | BlockFlags::GREEDY_MESHABLE
        | BlockFlags::COLLIDABLE
        | BlockFlags::DOES_SPAWN
        | BlockFlags::CAN_SUPPORT
);

const FOLIAGE: LazyLock<BlockFlags> = LazyLock::new(||
    BlockFlags::TRANSLUCENT
);

pub fn bootstrap_blocks() -> BlockRecord {
    let mut commit = BlockCommit::new();


    register_block(
        &mut commit,
        "grass",
        BlockBehaviour::new()
            .hardness(0.0)
            .toughness(0.0)
            .flags(*FOLIAGE)
            .build(),
        Vec::new()
    );

    register_block(
        &mut commit,
        "poppy",
        BlockBehaviour::new()
            .hardness(0.0)
            .toughness(0.0)
            .flags(*FOLIAGE)
            .build(),
        Vec::new()
    );

    register_block(
        &mut commit,
        "grass_block",
        BlockBehaviour::new()
            .hardness(0.65)
            .toughness(0.65)
            .flags(*FULL_BLOCK)
            .build(),
        vec![FULL_SHAPE]
    );

    register_block(
        &mut commit,
        "dirt",
        BlockBehaviour::new()
            .hardness(0.5)
            .toughness(0.5)
            .flags(*FULL_BLOCK)
            .build(),
        vec![FULL_SHAPE]
    );

    register_block(
        &mut commit,
        "sand",
        BlockBehaviour::new()
            .hardness(0.5)
            .toughness(0.5)
            .flags(*FULL_BLOCK)
            .build(),
        vec![FULL_SHAPE]
    );

    register_block(
        &mut commit,
        "stone",
        BlockBehaviour::new()
            .hardness(2.0)
            .toughness(6.0)
            .flags(*FULL_BLOCK)
            .build(),
        vec![FULL_SHAPE]
    );

    register_block(
        &mut commit,
        "bedrock",
        BlockBehaviour::new()
            .hardness(f32::INFINITY)
            .toughness(f32::INFINITY)
            .flags(BlockFlags::COLLIDABLE
                | BlockFlags::GREEDY_MESHABLE
                | BlockFlags::OCCLUDABLE
                | BlockFlags::CAN_SUPPORT
            )
            .build(),
        vec![FULL_SHAPE]
    );

    register_block(
        &mut commit,
        "oak_log",
        BlockBehaviour::new()
            .hardness(2.0)
            .toughness(2.0)
            .flags(*FULL_BLOCK)
            .build(),
        vec![FULL_SHAPE]
    );

    register_block(
        &mut commit,
        "oak_planks",
        BlockBehaviour::new()
            .hardness(2.0)
            .toughness(3.0)
            .flags(*FULL_BLOCK)
            .build(),
        vec![FULL_SHAPE]
    );

    register_block(
        &mut commit,
        "oak_planks_slab",
        BlockBehaviour::new()
            .hardness(2.0)
            .toughness(3.0)
            .flags(BlockFlags::COLLIDABLE
                | BlockFlags::OCCLUDABLE
                | BlockFlags::CAN_SUPPORT
            )
            .build(),
        vec![HALF_SHAPE]
    );

    register_block(
        &mut commit,
        "oak_planks_stair",
        BlockBehaviour::new()
            .hardness(2.0)
            .toughness(3.0)
            .flags(BlockFlags::COLLIDABLE
                | BlockFlags::OCCLUDABLE
                | BlockFlags::CAN_SUPPORT
            )
            .build(),
        vec![
            HALF_SHAPE,
            Aabb3d::from_min_max([0.0, 4.0, 0.0], [8.0, 8.0, 4.0])
        ]
    );

    register_block(
        &mut commit,
        "oak_planks_trapdoor",
        BlockBehaviour::new()
            .hardness(2.0)
            .toughness(3.0)
            .flags(BlockFlags::COLLIDABLE
                | BlockFlags::OCCLUDABLE
                | BlockFlags::CAN_SUPPORT
            )
            .build(),
        vec![Aabb3d::from_min_max([0.0, 0.0, 0.0], [8.0, 1.0, 8.0])]
    );

    register_block(
        &mut commit,
        "oak_leaves",
        BlockBehaviour::new()
            .hardness(0.2)
            .toughness(0.2)
            .flags(BlockFlags::COLLIDABLE
                | BlockFlags::CAN_SUPPORT
                | BlockFlags::GREEDY_MESHABLE
            )
            .build(),
        vec![FULL_SHAPE]
    );

    register_block(
        &mut commit,
        "snow_block",
        BlockBehaviour::new()
            .hardness(0.2)
            .toughness(0.2)
            .flags(*FULL_BLOCK)
            .build(),
        vec![FULL_SHAPE]
    );

    BlockRecord::finish(commit)
}

fn register_block(
    commit: &mut BlockCommit,
    name: &str,
    behaviour: BlockBehaviour,
    shapes: Vec<Aabb3d>,
) {
    commit.push(
        AssetLocation::with_default_namespace(name),
        Block::new()
            .behaviour(behaviour)
            .shapes(shapes)
            .build()
    )
}

fn register_block_with_attached(
    commit: &mut BlockCommit,
    name: &str,
    behaviour: BlockBehaviour,
    shapes: Vec<Aabb3d>,
    attached: Attachments
) {
    commit.push(
        AssetLocation::with_default_namespace(name),
        Block::new()
            .behaviour(behaviour)
            .shapes(shapes)
            .attachments(attached)
            .build()
    )
}