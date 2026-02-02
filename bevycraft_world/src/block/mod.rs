use crate::prelude::*;

pub mod block;
pub mod block_state;
pub mod state_definition;

pub const GRASS_BLOCK: Block = Block::new(
    BlockBehaviour::builder()
        .hardness(0.65)
        .toughness(0.65)
        .build()
);

pub const DIRT_BLOCK: Block = Block::new(
    BlockBehaviour::builder()
        .hardness(0.5)
        .toughness(0.5)
        .build()
);

pub const STONE_BLOCK: Block = Block::new(
    BlockBehaviour::builder()
        .hardness(2.0)
        .toughness(6.0)
        .build()
);

pub const COBBLESTONE_BLOCK: Block = Block::new(
    BlockBehaviour::builder()
        .hardness(2.0)
        .toughness(6.0)
        .build()
);

pub const OAK_LOG_BLOCK: Block = Block::new(
    BlockBehaviour::builder()
        .hardness(2.0)
        .toughness(2.0)
        .build()
);

pub const OAK_PLANK_BLOCK: Block = Block::new(
    BlockBehaviour::builder()
        .hardness(2.0)
        .toughness(3.0)
        .build()
);

pub const OAK_LEAVES_BLOCK: Block = Block::new(
    BlockBehaviour::builder()
        .hardness(0.2)
        .toughness(0.0)
        .occludable(false)
        .build()
);

pub const BEDROCK_BLOCK: Block = Block::new(
    BlockBehaviour::builder()
        .hardness(f32::INFINITY)
        .toughness(f32::INFINITY)
        .build()
);