use {
    crate::{
        consts::{FULL_BLOCK, FULL_SHAPE, SLAB_SHAPE, STAIR_SHAPE, TRAPDOOR_SHAPE},
        prelude::*,
    },
    bevycraft_macros::*,
};

context! {
    pub static AIR: Block = "air" => || Block::default();

    pub static GRASS_BLOCK: Block = "grass_block" => || Block::new()
        .behaviour(
            BlockBehaviour::new()
                .hardness(0.65)
                .toughness(0.65)
                .flags(*FULL_BLOCK)
                .build()
        )
        .shape(FULL_SHAPE)
        .build();

    pub static DIRT: Block = "dirt" => || Block::new()
        .behaviour(
            BlockBehaviour::new()
                .hardness(0.5)
                .toughness(0.5)
                .flags(*FULL_BLOCK)
                .build()
        )
        .shape(FULL_SHAPE)
        .build();

    pub static SAND: Block = "sand" => || Block::new()
        .behaviour(
            BlockBehaviour::new()
                .hardness(0.5)
                .toughness(0.5)
                .flags(*FULL_BLOCK)
                .build()
        )
        .shape(FULL_SHAPE)
        .build();
    
    pub static GRAVEL: Block = "gravel" => || Block::new()
        .behaviour(
            BlockBehaviour::new()
                .hardness(0.5)
                .toughness(0.5)
                .flags(*FULL_BLOCK)
                .build()
        )
        .shape(FULL_SHAPE)
        .build();
    
    pub static WATER: Block = "water" => || Block::new()
        .behaviour(
            BlockBehaviour::new()
                .hardness(0.0)
                .toughness(0.0)
                .flags(*FULL_BLOCK)
                .build()
        )
        .shape(BlockShape::None)
        .build();

    pub static SNOW_BLOCK: Block = "snow_block" => || Block::new()
        .behaviour(
            BlockBehaviour::new()
                .hardness(0.2)
                .toughness(0.2)
                .flags(*FULL_BLOCK)
                .build()
        )
        .shape(FULL_SHAPE)
        .build();

    pub static STONE: Block = "stone" => || Block::new()
        .behaviour(
            BlockBehaviour::new()
                .hardness(2.0)
                .toughness(6.0)
                .flags(*FULL_BLOCK)
                .build()
        )
        .shape(FULL_SHAPE)
        .build();

    pub static COBBLESTONE: Block = "cobblestone" => || Block::new()
        .behaviour(
            BlockBehaviour::new()
                .hardness(2.0)
                .toughness(6.0)
                .flags(*FULL_BLOCK)
                .build()
        )
        .shape(FULL_SHAPE)
        .build();

    pub static BEDROCK: Block = "bedrock" => || Block::new()
        .behaviour(
            BlockBehaviour::new()
                .hardness(f32::INFINITY)
                .toughness(f32::INFINITY)
                .flags(BlockFlags::OCCLUDABLE | BlockFlags::COLLIDABLE | BlockFlags::CAN_SUPPORT)
                .build()
        )
        .shape(FULL_SHAPE)
        .build();

    pub static OAK_LOG: Block = "oak_log" => || Block::new()
        .behaviour(
            BlockBehaviour::new()
                .hardness(2.0)
                .toughness(2.0)
                .flags(*FULL_BLOCK)
                .build()
        )
        .shape(FULL_SHAPE)
        .build();

    pub static OAK_PLANKS: Block = "oak_planks" => || Block::new()
        .behaviour(
            BlockBehaviour::new()
                .hardness(2.0)
                .toughness(3.0)
                .flags(*FULL_BLOCK)
                .build()
        )
        .shape(FULL_SHAPE)
        .build();

    pub static OAK_PLANKS_SLAB: Block = "oak_planks_slab" => || Block::new()
        .behaviour(
            BlockBehaviour::new()
                .hardness(2.0)
                .toughness(3.0)
                .flags(BlockFlags::COLLIDABLE | BlockFlags::OCCLUDABLE | BlockFlags::CAN_SUPPORT)
                .build()
        )
        .shape(SLAB_SHAPE)
        .build();

    pub static OAK_PLANKS_STAIR: Block = "oak_planks_stair" => || Block::new()
        .behaviour(
            BlockBehaviour::new()
                .hardness(2.0)
                .toughness(3.0)
                .flags(BlockFlags::COLLIDABLE | BlockFlags::OCCLUDABLE | BlockFlags::CAN_SUPPORT)
                .build()
        )
        .shape(STAIR_SHAPE)
        .build();

    pub static OAK_TRAPDOOR: Block = "oak_trapdoor" => || Block::new()
        .behaviour(
            BlockBehaviour::new()
                .hardness(2.0)
                .toughness(3.0)
                .flags(BlockFlags::COLLIDABLE | BlockFlags::OCCLUDABLE)
                .build()
        )
        .shape(TRAPDOOR_SHAPE)
        .build();

    pub static OAK_LEAVES: Block = "oak_leaves" => || Block::new()
        .behaviour(
            BlockBehaviour::new()
                .hardness(0.2)
                .toughness(0.2)
                .flags(BlockFlags::COLLIDABLE | BlockFlags::OCCLUDABLE)
                .build()
        )
        .shape(FULL_SHAPE)
        .build();

    pub static GRASS: Block = "grass" => || Block::new()
        .behaviour(
            BlockBehaviour::new()
                .hardness(0.0)
                .toughness(0.0)
                .flags(BlockFlags::empty())
                .build()
        )
        .build();

    pub static POPPY: Block = "poppy" => || Block::new()
        .behaviour(
            BlockBehaviour::new()
                .hardness(0.0)
                .toughness(0.0)
                .flags(BlockFlags::empty())
                .build()
        )
        .build();
}
