use bevy::log::info;
use simdnoise::NoiseBuilder;
use bevycraft_core::prelude::{AssetLocation, Record};
use crate::prelude::*;

pub struct BasicGenerator {
    pub seed: u32,
    pub frequency: f32,
    pub octaves: u8,
    pub amplitude_min: f32,
    pub amplitude_max: f32,
}

impl WorldGenerator for BasicGenerator {
    #[inline(always)]
    fn seed(&self) -> u32 {
        self.seed
    }

    #[inline(always)]
    fn generate_base_terrain(
        &self,
        chunk: &mut Chunk,
        blocks: &BlockRecord
    ) {
        let stone_id = blocks.key_to_idx(&AssetLocation::with_default_namespace("stone"))
            .unwrap() as u32;

        let dirt_id = blocks.key_to_idx(&AssetLocation::with_default_namespace("dirt"))
            .unwrap() as u32;

        let grass_block_id = blocks.key_to_idx(&AssetLocation::with_default_namespace("grass_block"))
            .unwrap() as u32;

        let world_pos = chunk.world_pos();

        let (noise, min, max) = NoiseBuilder::fbm_2d_offset(
            world_pos.x as f32, SECTION_SIZE as usize,
            world_pos.y as f32, SECTION_SIZE as usize,
        )
            .with_seed(self.seed as i32)
            .with_freq(self.frequency)
            .with_octaves(self.octaves)
            .generate();

        for x in 0..SECTION_SIZE {
            for z in 0..SECTION_SIZE {
                let index = x + (z * SECTION_SIZE);

                let surface_weight = (
                    self.amplitude_min +
                    (noise[index as usize] + 1.0) * (self.amplitude_max - self.amplitude_min)) as i32;

                for y in 0..64 {
                    let block = if y > surface_weight {
                        continue;
                    } else if y == surface_weight {
                        grass_block_id
                    } else if y > surface_weight - 3 {
                        dirt_id
                    } else {
                        stone_id
                    };

                    chunk.set_at([x as i32, y, z as i32], block);
                }
            }
        }
    }
}