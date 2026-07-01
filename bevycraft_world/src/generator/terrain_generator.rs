use {
    crate::prelude::{Chunk, ChunkGenerator, ChunkPos, CHUNK_SIZE},
    bevycraft_core::blocks::*,
    simdnoise::NoiseBuilder,
};

const WARP_MARGIN: usize = 32;
const ELEV_MAP_DIM: usize = (CHUNK_SIZE as usize) + 2 * WARP_MARGIN;

const FBM_ELEV_AMP: f32 = 1.97;
const FBM_CONTINENT_AMP: f32 = 1.94;
const FBM_TEMP_AMP: f32 = 1.75;
const FBM_HUMIDITY_AMP: f32 = 1.75;
const FBM_WARP_AMP: f32 = 1.75;

struct ColumnSample {
    continentality: f32,
    temperature: f32,
    humidity: f32,
    surface_height: i32,
}

#[derive(Debug, Clone)]
pub struct TerrainGenerator {
    pub seed: i32,

    pub continent_freq: f32,
    pub continent_octaves: u8,
    pub continent_gain: f32,
    pub continent_lacunarity: f32,

    pub temperature_freq: f32,
    pub temperature_octaves: u8,

    pub humidity_freq: f32,
    pub humidity_octaves: u8,

    pub elevation_freq: f32,
    pub elevation_octaves: u8,
    pub elevation_gain: f32,
    pub elevation_lacunarity: f32,

    pub warp_freq: f32,
    pub warp_octaves: u8,
    pub warp_strength: f32,

    pub amplitude_min: f32,
    pub amplitude_max: f32,
    pub sea_level: i32,
    pub beach_depth: i32,
    pub dirt_depth: i32,
    pub mountain_level: i32,
    pub snow_level: i32,
}

impl Default for TerrainGenerator {
    fn default() -> Self {
        Self {
            seed: 0,
            continent_freq: 0.0007,
            continent_octaves: 5,
            continent_gain: 0.5,
            continent_lacunarity: 2.0,
            temperature_freq: 0.0005,
            temperature_octaves: 3,
            humidity_freq: 0.0005,
            humidity_octaves: 3,
            elevation_freq: 0.008,
            elevation_octaves: 6,
            elevation_gain: 0.5,
            elevation_lacunarity: 2.0,
            warp_freq: 0.008,
            warp_octaves: 3,
            warp_strength: 28.0,
            amplitude_min: -60.0,
            amplitude_max: 200.0,
            sea_level: 64,
            beach_depth: 4,
            dirt_depth: 3,
            mountain_level: 110,
            snow_level: 150,
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

    #[inline]
    fn climate_pass(&self, wx: f32, wz: f32) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
        let s = CHUNK_SIZE as usize;

        let (continent, ..) = NoiseBuilder::fbm_2d_offset(wx, s, wz, s)
            .with_seed(self.seed)
            .with_octaves(self.continent_octaves)
            .with_freq(self.continent_freq)
            .with_gain(self.continent_gain)
            .with_lacunarity(self.continent_lacunarity)
            .generate();

        let (temp, ..) = NoiseBuilder::fbm_2d_offset(wx, s, wz, s)
            .with_seed(self.seed.wrapping_add(0x1111_1111))
            .with_octaves(self.temperature_octaves)
            .with_freq(self.temperature_freq)
            .generate();

        let (humidity, ..) = NoiseBuilder::fbm_2d_offset(wx, s, wz, s)
            .with_seed(self.seed.wrapping_add(0x2222_2222))
            .with_octaves(self.humidity_octaves)
            .with_freq(self.humidity_freq)
            .generate();

        (continent, temp, humidity)
    }

    #[inline]
    fn warp_pass(&self, wx: f32, wz: f32) -> (Vec<f32>, Vec<f32>) {
        let s = CHUNK_SIZE as usize;

        let (dx, ..) = NoiseBuilder::fbm_2d_offset(wx, s, wz, s)
            .with_seed(self.seed.wrapping_add(0x3333_3333))
            .with_octaves(self.warp_octaves)
            .with_freq(self.warp_freq)
            .generate();

        let (dz, ..) = NoiseBuilder::fbm_2d_offset(wx, s, wz, s)
            .with_seed(self.seed.wrapping_add(0x4444_4444))
            .with_octaves(self.warp_octaves)
            .with_freq(self.warp_freq)
            .generate();

        (dx, dz)
    }

    #[inline]
    fn elevation_pass(&self, wx: f32, wz: f32) -> Vec<f32> {
        let m = WARP_MARGIN as f32;
        let (elev, ..) = NoiseBuilder::fbm_2d_offset(wx - m, ELEV_MAP_DIM, wz - m, ELEV_MAP_DIM)
            .with_seed(self.seed.wrapping_add(0x5555_5555))
            .with_octaves(self.elevation_octaves)
            .with_freq(self.elevation_freq)
            .with_gain(self.elevation_gain)
            .with_lacunarity(self.elevation_lacunarity)
            .generate();
        elev
    }

    #[inline]
    fn sample_warped(
        elev: &[f32],
        local_x: f32,
        local_z: f32,
        dx: f32,
        dz: f32,
        strength: f32,
    ) -> f32 {
        let m = WARP_MARGIN as f32;
        let max = (ELEV_MAP_DIM - 2) as f32;

        let ex = (local_x + dx * strength + m).clamp(0.0, max);
        let ez = (local_z + dz * strength + m).clamp(0.0, max);

        let x0 = ex as usize;
        let z0 = ez as usize;
        let fx = ex.fract();
        let fz = ez.fract();

        let d = ELEV_MAP_DIM;
        let v00 = elev[z0 * d + x0];
        let v10 = elev[z0 * d + x0 + 1];
        let v01 = elev[(z0 + 1) * d + x0];
        let v11 = elev[(z0 + 1) * d + x0 + 1];

        let h0 = v00 + (v10 - v00) * fx;
        let h1 = v01 + (v11 - v01) * fx;
        h0 + (h1 - h0) * fz
    }

    #[inline]
    fn to_surface_height(&self, noise: f32, continent: f32) -> i32 {
        let n = normalize(noise, FBM_ELEV_AMP);
        let cont = normalize_signed(continent, FBM_CONTINENT_AMP);

        let sl = self.sea_level as f32;
        let a_max = self.amplitude_max;
        let a_min = self.amplitude_min;

        let height = if cont < -0.1 {
            let depth = ((-cont - 0.1) / 0.9).clamp(0.0, 1.0);
            sl - depth * 50.0 + n * 15.0
        } else if cont < 0.15 {
            let t = (cont + 0.1) / 0.25;
            let low = sl - 10.0 + n * 15.0;
            let high = sl + n * 60.0;
            low + (high - low) * t
        } else {
            let range = a_max - sl;
            let scale = ((cont - 0.15) / 0.85).clamp(0.0, 1.0);
            sl + n * range * (0.3 + 0.7 * scale)
        };

        height.clamp(a_min, a_max) as i32
    }

    #[inline]
    fn decorate_column(&self, chunk: &mut Chunk, lx: i32, lz: i32, wy: i32, col: &ColumnSample) {
        let surf = col.surface_height;
        let is_ocean = surf < self.sea_level;
        let is_beach = !is_ocean && surf < self.sea_level + self.beach_depth;
        let is_mount = !is_ocean && surf >= self.mountain_level;
        let is_snowy = !is_ocean && surf >= self.snow_level;

        for ly in 0..CHUNK_SIZE {
            let world_y = wy + ly;

            let block = if is_ocean {
                self.ocean_block(world_y, surf, col)
            } else {
                self.land_block(world_y, surf, lx, lz, is_beach, is_mount, is_snowy)
            };

            if let Some(id) = block {
                chunk.set([lx, ly, lz], id);
            }
        }
    }

    #[inline]
    fn ocean_block(&self, wy: i32, surf: i32, col: &ColumnSample) -> Option<usize> {
        if wy > self.sea_level {
            return None;
        }
        if wy > surf {
            return Some(*WATER);
        }

        let depth = surf - wy;
        Some(match depth {
            0 => {
                if col.continentality < -0.5 {
                    *GRAVEL
                } else {
                    *SAND
                }
            }
            d if d < self.dirt_depth => *GRAVEL,
            _ => *STONE,
        })
    }

    #[inline]
    fn land_block(
        &self,
        wy: i32,
        surf: i32,
        lx: i32,
        lz: i32,
        is_beach: bool,
        is_mount: bool,
        is_snowy: bool,
    ) -> Option<usize> {
        let depth = surf - wy;

        if depth < 0 {
            if wy == surf + 1 && !is_beach && !is_mount && !is_snowy {
                return self.surface_decoration(lx, lz, surf);
            }
            return None;
        }

        Some(match depth {
            0 => {
                if is_snowy {
                    *SNOW_BLOCK
                } else if is_mount {
                    *STONE
                } else if is_beach {
                    *SAND
                } else {
                    *GRASS_BLOCK
                }
            }
            d if d < self.dirt_depth => {
                if is_beach {
                    *SAND
                } else {
                    *DIRT
                }
            }
            _ => *STONE,
        })
    }

    #[inline]
    fn surface_decoration(&self, lx: i32, lz: i32, surf: i32) -> Option<usize> {
        let h = (lx as u32)
            .wrapping_mul(2_654_435_761)
            .wrapping_add((lz as u32).wrapping_mul(2_246_822_519))
            .wrapping_add(surf as u32)
            .wrapping_mul(1_000_000_007);

        match h % 100 {
            0..=9 => Some(*GRASS),
            10..=12 => Some(*POPPY),
            _ => None,
        }
    }
}

impl ChunkGenerator for TerrainGenerator {
    fn generate(&self, chunk_pos: ChunkPos) -> Chunk {
        let mut chunk = Chunk::empty();

        let wx = (chunk_pos.x * CHUNK_SIZE) as f32;
        let wy = chunk_pos.y * CHUNK_SIZE;
        let wz = (chunk_pos.z * CHUNK_SIZE) as f32;

        let (continent_map, temp_map, humidity_map) = self.climate_pass(wx, wz);

        let (warp_dx, warp_dz) = self.warp_pass(wx, wz);

        let elevation_map = self.elevation_pass(wx, wz);

        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                let idx = (z * CHUNK_SIZE + x) as usize;

                let continent = continent_map[idx];

                let dx = normalize_signed(warp_dx[idx], FBM_WARP_AMP);
                let dz = normalize_signed(warp_dz[idx], FBM_WARP_AMP);

                let raw_elev = Self::sample_warped(
                    &elevation_map,
                    x as f32,
                    z as f32,
                    dx,
                    dz,
                    self.warp_strength,
                );

                let surf = self.to_surface_height(raw_elev, continent_map[idx]);

                let raw_temp = normalize(temp_map[idx], FBM_TEMP_AMP);
                let alt_factor = ((surf - self.sea_level) as f32 / 100.0).max(0.0);
                let temperature = (raw_temp - alt_factor * 0.4).clamp(0.0, 1.0);

                let humidity = normalize(humidity_map[idx], FBM_HUMIDITY_AMP);

                let col = ColumnSample {
                    continentality: continent,
                    temperature,
                    humidity,
                    surface_height: surf,
                };

                self.decorate_column(&mut chunk, x, z, wy, &col);
            }
        }

        chunk
    }
}

#[inline]
const fn normalize(v: f32, amp: f32) -> f32 {
    (v / amp).clamp(-1.0, 1.0) * 0.5 + 0.5
}

#[inline]
const fn normalize_signed(v: f32, amp: f32) -> f32 {
    (v / amp).clamp(-1.0, 1.0)
}
