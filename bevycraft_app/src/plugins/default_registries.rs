use phf::phf_ordered_map;
use bevycraft_core::prelude::*;
use bevycraft_world::prelude::*;
use bevycraft_world::presets::*;
use crate::prelude::*;

pub static BLOCKS: CompiledRegistry<Block> = CompiledRegistry::new(
    GameRegistries::DEFAULT_NAMESPACE,
    phf_ordered_map!(
        "stone" => STONE_BLOCK,
        "cobblestone" => COBBLESTONE_BLOCK,
        "grass" => GRASS_BLOCK,
        "dirt" => DIRT_BLOCK,
        "bedrock" => BEDROCK_BLOCK,
        "oak_log" => OAK_LOG_BLOCK,
        "oak_plank" => OAK_PLANK_BLOCK,
        "oak_leaves" => OAK_LEAVES_BLOCK,
    ),
);