use {
    crate::prelude::{BlockModel, Direction, ModelCache},
    bevy::prelude::IVec3,
    bevycraft_core::prelude::Block,
    bevycraft_world::prelude::{ChunkMap, ChunkPos, ChunkStorage, CHUNK_SIZE},
    std::sync::Arc,
};

#[derive(Clone)]
pub struct MeshInput {
    _pos: ChunkPos,
    storage: Arc<ChunkStorage>,
    neighbors: [Option<Arc<ChunkStorage>>; 6],
    model_cache: ModelCache<Block, BlockModel>,
}

impl MeshInput {
    pub fn build(
        pos: ChunkPos,
        storage: Arc<ChunkStorage>,
        chunk_map: &ChunkMap,
        model_cache: ModelCache<Block, BlockModel>,
    ) -> Self {
        let neighbors = Direction::ALL.map(|dir| {
            let nb_pos = ChunkPos::from(pos + dir.offset());

            chunk_map.get(&nb_pos).map(|nb| nb.storage.clone())
        });

        Self {
            _pos: pos,
            storage,
            neighbors,
            model_cache,
        }
    }

    #[inline]
    pub fn get_model_at(&self, pos: IVec3) -> Option<&BlockModel> {
        let id = self.get_id_at(pos);

        self.get_model_of(id)
    }

    #[inline]
    pub fn get_model_of(&self, id: usize) -> Option<&BlockModel> {
        self.model_cache.get(id)
    }

    #[inline]
    pub fn get_id_at(&self, pos: IVec3) -> usize {
        if in_bounds(pos) {
            return self.storage.get(pos);
        }

        let dir = get_direction(pos).unwrap();

        self.neighbors[dir as usize]
            .as_ref()
            .map(|nb| nb.get(pos.rem_euclid(IVec3::splat(CHUNK_SIZE))))
            .unwrap_or(0)
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.storage.is_empty()
    }
}

#[inline(always)]
const fn get_direction(pos: IVec3) -> Option<Direction> {
    let out_x = pos.x < 0 || pos.x >= CHUNK_SIZE;
    let out_y = pos.y < 0 || pos.y >= CHUNK_SIZE;
    let out_z = pos.z < 0 || pos.z >= CHUNK_SIZE;

    match (out_x, out_y, out_z) {
        (true, false, false) => {
            if pos.x < 0 {
                Some(Direction::NegX)
            } else {
                Some(Direction::PosX)
            }
        }
        (false, true, false) => {
            if pos.y < 0 {
                Some(Direction::NegY)
            } else {
                Some(Direction::PosY)
            }
        }
        (false, false, true) => {
            if pos.z < 0 {
                Some(Direction::NegZ)
            } else {
                Some(Direction::PosZ)
            }
        }
        _ => None,
    }
}

#[inline(always)]
const fn in_bounds(pos: IVec3) -> bool {
    pos.x >= 0
        && pos.x < CHUNK_SIZE
        && pos.y >= 0
        && pos.y < CHUNK_SIZE
        && pos.z >= 0
        && pos.z < CHUNK_SIZE
}
