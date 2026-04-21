use std::sync::Arc;
use bevy::prelude::IVec2;
use fastrand::Rng;
use simdnoise::NoiseBuilder;
use bevycraft_core::prelude::{AssetLocation, Record};
use crate::prelude::*;

pub struct BasicGenerator {
    pub seed: u32,
    pub frequency: f32,
    pub octaves: u8,
    pub amplitude_min: f32,
    pub amplitude_max: f32,
    pub min_height: i32,
    pub max_height: i32,
    pub snow_height: i32,
}

impl WorldGenerator for BasicGenerator {
    #[inline(always)]
    fn seed(&self) -> u32 {
        self.seed
    }

    #[inline(always)]
    fn generate_base_terrain(
        &self,
        position: IVec2,
        chunk   : &mut Chunk,
        blocks  : Arc<BlockRecord>
    ) {
        let stone_id = blocks.key_to_idx(&AssetLocation::with_default_namespace("stone"))
            .unwrap();

        let dirt_id = blocks.key_to_idx(&AssetLocation::with_default_namespace("dirt"))
            .unwrap();

        let grass_block_id = blocks.key_to_idx(&AssetLocation::with_default_namespace("grass_block"))
            .unwrap();

        let snow_block_id = blocks.key_to_idx(&AssetLocation::with_default_namespace("snow_block"))
            .unwrap();

        let world_pos = IVec2::new(
            position.x * SECTION_SIZE,
            position.y * SECTION_SIZE,
        );

        let (noise, _, _) = NoiseBuilder::fbm_2d_offset(
            world_pos.x as f32, SECTION_SIZE as usize,
            world_pos.y as f32, SECTION_SIZE as usize,
        )
            .with_seed(self.seed as i32)
            .with_octaves(self.octaves)
            .with_freq(self.frequency)
            .generate();

        let (snow, _, _) = NoiseBuilder::cellular_2d_offset(
            world_pos.x as f32, SECTION_SIZE as usize,
            world_pos.y as f32, SECTION_SIZE as usize,
        )
            .with_seed(self.seed as i32)
            .with_freq(self.frequency)
            .generate();

        let height_range = self.amplitude_max - self.amplitude_min;

        for x in 0..SECTION_SIZE {
            for z in 0..SECTION_SIZE {
                let index = x + (z * SECTION_SIZE);

                let normalized_noise = (noise[index as usize] + 1.0) * 0.5;

                let surface_weight = (self.amplitude_min + (normalized_noise * height_range)) as i32;

                let variation = snow[index as usize];

                let local_snow_height = self.snow_height as f32 + (variation * 16.0);

                for y in self.min_height..self.max_height {
                    let block = if y > surface_weight {
                        continue;
                    } else if y == surface_weight {
                        if y as f32 > local_snow_height {
                            snow_block_id
                        } else {
                            grass_block_id
                        }
                    } else if y > surface_weight - 3 {
                        dirt_id
                    } else {
                        stone_id
                    };

                    chunk.set_at([x, y, z], block);
                }
            }
        }
    }

    fn generate_features(
        &self,
        _position: IVec2,
        chunk   : &mut Chunk,
        blocks  : Arc<BlockRecord>
    ) {
        let grass_block_id = blocks.key_to_idx(&AssetLocation::with_default_namespace("grass_block"))
            .unwrap();

        let grass_id = blocks.key_to_idx(&AssetLocation::with_default_namespace("grass"))
            .unwrap();

        let poppy_id = blocks.key_to_idx(&AssetLocation::with_default_namespace("poppy"))
            .unwrap();

        let mut rng = Rng::with_seed(self.seed as u64);

        for x in 0..SECTION_SIZE {
            for z in 0..SECTION_SIZE {
                let mut surface_y = -1;

                for y in (self.min_height..self.max_height).rev() {
                    if let Some(_) = chunk.get_at([x, y, z]) {
                        surface_y = y;
                        break;
                    }
                }

                if surface_y == -1 || surface_y >= self.max_height - 1 {
                    continue;
                }

                if chunk.get_at([x, surface_y, z]).unwrap() == grass_block_id {
                    let change = rng.f32();

                    if change < 0.035 {
                        chunk.set_at([x, surface_y + 1, z], poppy_id);
                    } else if change < 0.2 {
                        chunk.set_at([x, surface_y + 1, z], grass_id);
                    }
                }
            }
        }
    }
}