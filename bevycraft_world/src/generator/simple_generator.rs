use bevycraft_core::prelude::{AssetLocation, BlockType, CoreRegistries};
use simdnoise::NoiseBuilder;

use crate::prelude::{CHUNK_SIZE, Chunk, ChunkGenerator, ChunkPos};

pub struct SimpleGenerator {
    pub seed: i32,
    pub amplitude_min: f32,
    pub amplitude_max: f32,
    pub octaves: u8,
    pub frequency: f32,
    pub gain: f32,
    pub lacunarity: f32,
}

impl Default for SimpleGenerator {
    #[inline]
    fn default() -> Self {
        SimpleGenerator {
            seed: 5,
            amplitude_min: 0.0,
            amplitude_max: 128.0,
            octaves: 7,
            frequency: 0.03,
            gain: 2.0,
            lacunarity: 0.5,
        }
    }
}

impl ChunkGenerator for SimpleGenerator {
    #[inline]
    fn seed(&self) -> i32 {
        self.seed
    }

    #[inline]
    fn fill(&self, position: ChunkPos, chunk: &mut Chunk) {
        let [grass, dirt, stone] = [
            get_block_type("grass_block"),
            get_block_type("dirt"),
            get_block_type("stone"),
        ];

        let world_pos = position.into_world_pos();

        let (noise_2d, _, _) = NoiseBuilder::fbm_2d_offset(
            world_pos.x,
            CHUNK_SIZE as usize,
            world_pos.z,
            CHUNK_SIZE as usize,
        )
        .with_seed(self.seed)
        .with_octaves(self.octaves)
        .with_freq(self.frequency)
        .with_gain(self.gain)
        .with_lacunarity(self.lacunarity)
        .generate();

        let world_height = world_pos.y as i32;

        let height_range = self.amplitude_max - self.amplitude_min;

        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let noise = (noise_2d[(x + (z * CHUNK_SIZE)) as usize] + 1.0) * 0.5;

                let surface_height = (self.amplitude_min + (noise * height_range)) as i32;

                for y in world_height..world_height + CHUNK_SIZE {
                    let block = if y > surface_height {
                        continue;
                    } else if y == surface_height {
                        grass
                    } else if y > surface_height - 3 {
                        dirt
                    } else {
                        stone
                    };

                    chunk.set([x, y.rem_euclid(CHUNK_SIZE), z], block);
                }
            }
        }
    }
}

#[inline(always)]
fn get_block_type(name: &'static str) -> BlockType {
    CoreRegistries::blocks()
        .key_to_type(&AssetLocation::parse(name))
        .unwrap()
}
