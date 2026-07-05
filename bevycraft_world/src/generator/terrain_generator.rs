use {
    crate::prelude::{Chunk, ChunkGenerator, ChunkPos, CHUNK_SIZE},
    bevycraft_core::blocks::*,
    simdnoise::NoiseBuilder,
};

const NOISE_OFFSET: f32 = 37.5;

const MAX_CONTINENT: f32 = 0.04;
const MAX_CLIMATE: f32 = 0.0375;
const MAX_WARP: f32 = 0.15;

const ELEV_GRID_MARGIN: f32 = 5.0;
const ELEV_GRID_SIZE: usize = CHUNK_SIZE as usize + 1 + (2 * ELEV_GRID_MARGIN as usize); // 27

#[derive(Debug, Clone, Copy, PartialEq)]
enum Biome {
    Ocean,
    Beach,
    Desert,
    Savanna,
    Plains,
    Forest,
    Jungle,
    Tundra,
    Taiga,
    Mountain,
}

struct ColumnSample {
    temperature: f32,
    surface_height: i32,
    biome: Biome,
}

#[derive(Debug, Clone)]
pub struct TerrainGenerator {
    pub seed: i32,
    pub freq: f32,
    pub octaves: u8,
    pub gain: f32,
    pub lacunarity: f32,
    pub continent_freq: f32,
    pub temperature_freq: f32,
    pub humidity_freq: f32,
    pub warp_freq: f32,
    pub warp_strength: f32,
    pub amplitude_min: f32,
    pub amplitude_max: f32,
    pub sea_level: i32,
    pub dirt_depth: i32,
    pub snow_line: i32,
    pub snow_cap_height: i32,
}

impl Default for TerrainGenerator {
    fn default() -> Self {
        Self {
            seed: 5,
            freq: 0.03,
            octaves: 7,
            gain: 2.0,
            lacunarity: 0.5,
            continent_freq: 0.008,
            temperature_freq: 0.015,
            humidity_freq: 0.015,
            warp_freq: 0.016,
            warp_strength: 5.0,
            amplitude_min: -60.0,
            amplitude_max: 256.0,
            sea_level: 64,
            dirt_depth: 3,
            snow_line: 140,
            snow_cap_height: 40,
        }
    }
}

impl TerrainGenerator {
    pub fn with_seed(seed: i32) -> Self {
        Self {
            seed,
            ..Self::default()
        }
    }

    fn climate_pass(&self, wx: f32, wz: f32) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
        let s = CHUNK_SIZE as usize;

        let (continent, ..) = NoiseBuilder::fbm_2d_offset(wx, s, wz, s)
            .with_seed(self.seed)
            .with_octaves(5)
            .with_freq(self.continent_freq)
            .with_gain(0.5)
            .with_lacunarity(2.0)
            .generate();

        let (temp, ..) = NoiseBuilder::fbm_2d_offset(wx, s, wz, s)
            .with_seed(self.seed.wrapping_add(0x1111_1111))
            .with_octaves(3)
            .with_freq(self.temperature_freq)
            .with_gain(0.5)
            .with_lacunarity(2.0)
            .generate();

        let (humidity, ..) = NoiseBuilder::fbm_2d_offset(wx, s, wz, s)
            .with_seed(self.seed.wrapping_add(0x2222_2222))
            .with_octaves(3)
            .with_freq(self.humidity_freq)
            .with_gain(0.5)
            .with_lacunarity(2.0)
            .generate();

        (continent, temp, humidity)
    }

    fn warp_pass(&self, wx: f32, wz: f32) -> (Vec<f32>, Vec<f32>) {
        let s = CHUNK_SIZE as usize;

        let (dx, ..) = NoiseBuilder::fbm_2d_offset(wx, s, wz, s)
            .with_seed(self.seed.wrapping_add(0x3333_3333))
            .with_octaves(3)
            .with_freq(self.warp_freq)
            .with_gain(self.gain)
            .with_lacunarity(self.lacunarity)
            .generate();

        let (dz, ..) = NoiseBuilder::fbm_2d_offset(wx, s, wz, s)
            .with_seed(self.seed.wrapping_add(0x4444_4444))
            .with_octaves(3)
            .with_freq(self.warp_freq)
            .with_gain(self.gain)
            .with_lacunarity(self.lacunarity)
            .generate();

        (dx, dz)
    }

    fn elevation_grid(&self, grid_wx: f32, grid_wz: f32) -> Vec<f32> {
        let s = ELEV_GRID_SIZE;
        let (elev, ..) = NoiseBuilder::fbm_2d_offset(grid_wx, s, grid_wz, s)
            .with_seed(self.seed.wrapping_add(0x5555_5555))
            .with_octaves(self.octaves)
            .with_freq(self.freq)
            .with_gain(self.gain)
            .with_lacunarity(self.lacunarity)
            .generate();
        elev
    }

    fn sample_grid(grid: &[f32], size: usize, x: f32, z: f32) -> f32 {
        let xi = (x.max(0.0) as usize).min(size - 2);
        let zi = (z.max(0.0) as usize).min(size - 2);
        let xf = x - xi as f32;
        let zf = z - zi as f32;

        let ix = |i: usize, j: usize| i + j * size;
        let v00 = grid[ix(xi, zi)];
        let v10 = grid[ix(xi + 1, zi)];
        let v01 = grid[ix(xi, zi + 1)];
        let v11 = grid[ix(xi + 1, zi + 1)];

        let v0 = v00 + (v10 - v00) * xf;
        let v1 = v01 + (v11 - v01) * xf;
        v0 + (v1 - v0) * zf
    }

    fn to_surface_height(&self, raw_elev: f32, continent: f32) -> i32 {
        let c = continent;
        let n = (raw_elev + 1.0) * 0.5;
        let sl = self.sea_level as f32;

        let height = if c < -0.1 {
            let depth = (-c - 0.1) / 0.9;
            (sl - depth * 50.0) + n * depth * 15.0 - 0.5
        } else if c < 0.15 {
            let t = (c + 0.1) / 0.25;
            sl + (n + 0.35) * 60.0 * t
        } else {
            let scale = (c - 0.15) / 0.85;
            let range = self.amplitude_max - sl;
            sl + (n + 0.35) * range * (0.3 + 0.7 * scale)
        };

        height as i32
    }

    fn biome(&self, temperature: f32, humidity: f32, surf: i32, continent: f32) -> Biome {
        if continent < -0.1 {
            return Biome::Ocean;
        }
        if continent > 0.25 {
            return Biome::Mountain;
        }
        if surf < self.sea_level {
            return Biome::Ocean;
        }
        if surf >= self.sea_level && surf < self.sea_level + 3 {
            return Biome::Beach;
        }
        match temperature {
            t if t < 0.2 => match humidity {
                h if h < 0.3 => Biome::Tundra,
                _ => Biome::Taiga,
            },
            t if t < 0.5 => match humidity {
                h if h < 0.3 => Biome::Savanna,
                h if h < 0.6 => Biome::Plains,
                _ => Biome::Forest,
            },
            _ => match humidity {
                h if h < 0.3 => Biome::Desert,
                h if h < 0.6 => Biome::Savanna,
                _ => Biome::Jungle,
            },
        }
    }

    fn decorate_column(
        &self,
        chunk: &mut Chunk,
        lx: i32,
        lz: i32,
        wy: i32,
        col: &ColumnSample,
        world_x: i32,
        world_z: i32,
    ) {
        let surf = col.surface_height;

        for ly in 0..CHUNK_SIZE {
            let world_y = wy + ly;
            let depth = surf - world_y;

            if depth < 0 {
                if col.biome == Biome::Ocean && world_y <= self.sea_level {
                    chunk.set([lx, ly, lz], *WATER);
                } else if world_y == surf + 1 {
                    if let Some(id) = self.surface_decoration(col, world_x, world_z) {
                        chunk.set([lx, ly, lz], id);
                    }
                }
                continue;
            }

            let block = match col.biome {
                Biome::Ocean => self.ocean_block(depth, col),
                _ => self.land_block(depth, world_y, col, world_x, world_z),
            };
            chunk.set([lx, ly, lz], block);
        }
    }

    fn ocean_block(&self, depth: i32, col: &ColumnSample) -> usize {
        match depth {
            0 => {
                if col.temperature < 0.3 {
                    *GRAVEL
                } else {
                    *SAND
                }
            }
            d if d < self.dirt_depth => *GRAVEL,
            _ => *STONE,
        }
    }

    fn land_block(
        &self,
        depth: i32,
        world_y: i32,
        col: &ColumnSample,
        world_x: i32,
        world_z: i32,
    ) -> usize {
        let needs_snow = matches!(col.biome, Biome::Mountain | Biome::Taiga)
            && depth == 0
            && world_y >= self.snow_line;

        if needs_snow {
            let coverage =
                ((world_y - self.snow_line) as f32 / self.snow_cap_height as f32).min(1.0);
            let seed = (world_x as u64)
                .wrapping_mul(3_747_613_93)
                .wrapping_add(world_z as u64)
                .wrapping_mul(6_682_652_63)
                .wrapping_add(world_y as u64)
                .wrapping_mul(1_274_126_177);
            let mut rng = fastrand::Rng::with_seed(seed);
            if rng.f32() < coverage {
                return *SNOW_BLOCK;
            }
        }

        match col.biome {
            Biome::Mountain => *STONE,
            Biome::Desert | Biome::Beach => match depth {
                0 => *SAND,
                d if d < self.dirt_depth => *SAND,
                _ => *STONE,
            },
            Biome::Tundra => match depth {
                0 => *SNOW_BLOCK,
                d if d < self.dirt_depth => *DIRT,
                _ => *STONE,
            },
            Biome::Taiga => match depth {
                0 => *GRASS_BLOCK,
                d if d < self.dirt_depth => *DIRT,
                _ => *STONE,
            },
            _ => match depth {
                0 => *GRASS_BLOCK,
                d if d < self.dirt_depth => *DIRT,
                _ => *STONE,
            },
        }
    }

    fn surface_decoration(&self, col: &ColumnSample, world_x: i32, world_z: i32) -> Option<usize> {
        match col.biome {
            Biome::Savanna | Biome::Plains | Biome::Forest | Biome::Jungle => {
                let seed = (world_x as u64)
                    .wrapping_mul(3_747_613_93)
                    .wrapping_add(world_z as u64)
                    .wrapping_mul(6_682_652_63)
                    .wrapping_add(col.surface_height as u64)
                    .wrapping_mul(1_274_126_177);
                let mut rng = fastrand::Rng::with_seed(seed);
                match rng.u8(0..100) {
                    0..=9 => Some(*GRASS),
                    10..=12 => Some(*POPPY),
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

impl ChunkGenerator for TerrainGenerator {
    fn generate(&self, chunk_pos: ChunkPos) -> Chunk {
        let mut chunk = Chunk::empty();

        let world_pos = chunk_pos.into_world_pos();
        let wx = world_pos.x + NOISE_OFFSET;
        let wy = world_pos.y;
        let wz = world_pos.z + NOISE_OFFSET;

        let (continent_map, temp_map, humidity_map) = self.climate_pass(wx, wz);
        let (warp_dx, warp_dz) = self.warp_pass(wx, wz);

        let elev_grid = self.elevation_grid(wx - ELEV_GRID_MARGIN, wz - ELEV_GRID_MARGIN);

        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let idx = (z * CHUNK_SIZE + x) as usize;

                let continent = continent_map[idx] / MAX_CONTINENT;
                let temperature = (temp_map[idx] / MAX_CLIMATE + 1.0) * 0.5;
                let humidity = (humidity_map[idx] / MAX_CLIMATE + 1.0) * 0.5;

                let du = warp_dx[idx] / MAX_WARP * self.warp_strength;
                let dv = warp_dz[idx] / MAX_WARP * self.warp_strength;
                let gx = ELEV_GRID_MARGIN + x as f32 + du;
                let gz = ELEV_GRID_MARGIN + z as f32 + dv;

                let raw_elev = Self::sample_grid(&elev_grid, ELEV_GRID_SIZE, gx, gz);
                let surf = self.to_surface_height(raw_elev, continent);

                let alt_factor = ((surf - self.sea_level) as f32 / 100.0).max(0.0);
                let biome_temp = temperature - alt_factor * 0.4;

                let biome = self.biome(biome_temp, humidity, surf, continent);

                let col = ColumnSample {
                    temperature: biome_temp,
                    surface_height: surf,
                    biome,
                };

                let world_x = chunk_pos.x * CHUNK_SIZE + x;
                let world_z = chunk_pos.z * CHUNK_SIZE + z;

                self.decorate_column(&mut chunk, x, z, wy as i32, &col, world_x, world_z);
            }
        }

        chunk
    }
}
