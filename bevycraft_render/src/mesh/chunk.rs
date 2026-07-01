use {
    crate::{
        mesh::{buffer::VertexBufferSet, input::MeshInput},
        prelude::{Direction, OcclusionMask},
    },
    bevy::prelude::{IVec3, Mesh},
    bevycraft_core::blocks::AIR,
    bevycraft_world::prelude::CHUNK_SIZE,
};

pub struct ChunkMeshOutput {
    pub opaque: Option<Mesh>,
    pub cutout: Option<Mesh>,
    pub translucent: Option<Mesh>,
}

impl ChunkMeshOutput {
    pub const EMPTY: Self = Self {
        opaque: None,
        cutout: None,
        translucent: None,
    };

    #[inline]
    pub fn try_from_set(set: VertexBufferSet) -> Self {
        Self {
            opaque: set.opaque.try_mesh(),
            cutout: set.cutout.try_mesh(),
            translucent: set.translucent.try_mesh(),
        }
    }
}

#[inline(always)]
pub fn mesh_chunk(input: MeshInput) -> ChunkMeshOutput {
    if input.is_empty() {
        return ChunkMeshOutput::EMPTY;
    };

    mesh(input)
}

#[inline(always)]
fn mesh(input: MeshInput) -> ChunkMeshOutput {
    let mut bufs = VertexBufferSet::default();

    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let local = IVec3::new(x, y, z);

                let id = input.get_id_at(local);

                if id == *AIR {
                    continue;
                };

                let Some(model) = input.get_model_of(id) else { continue };

                let offset = [x as f32, y as f32, z as f32];

                bufs.push_quads_with_offset(
                    model.iter_inner_quads(),
                    offset,
                    Some([0.2, 0.8, 0.2]),
                );

                for dir in Direction::ALL {
                    let nb_pos = local + dir.offset();
                    let nb_mask = sample_neighbor_mask(&input, nb_pos, dir);

                    if model.mask(dir).is_occluded_by(nb_mask) {
                        continue;
                    };

                    bufs.push_quads_with_offset(
                        model
                            .iter_outer_quads_at(dir)
                            .filter(|&q| !q.mask.is_occluded_by(nb_mask)),
                        offset,
                        Some([0.2, 0.8, 0.2]),
                    );
                }
            }
        }
    }

    ChunkMeshOutput::try_from_set(bufs)
}

#[inline(always)]
fn sample_neighbor_mask(input: &MeshInput, nb: IVec3, dir: Direction) -> OcclusionMask {
    let model = input.get_model_at(nb);

    model.map(|m| m.mask(dir)).unwrap_or(OcclusionMask::EMPTY)
}

#[inline(always)]
pub fn border_coords(nb: IVec3, s: i32) -> (i32, i32) {
    let x = nb.x.rem_euclid(s);
    let y = nb.y.rem_euclid(s);
    let z = nb.z.rem_euclid(s);

    if nb.x < 0 || nb.x >= s {
        (y, z)
    } else if nb.y < 0 || nb.y >= s {
        (x, z)
    } else {
        (x, y)
    }
}
