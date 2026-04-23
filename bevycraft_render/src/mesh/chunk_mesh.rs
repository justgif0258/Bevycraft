use std::sync::Arc;
use bevy::prelude::{Handle, IVec3, Mesh};
use bevycraft_world::prelude::{BlockRecord, BlockType, Chunk, SECTION_SIZE};
use crate::prelude::{ArrayTexture, BlockMeshCache, Facing, MeshBuffer, Quad, RenderMode};

pub struct ChunkMeshHandle {
    pub opaque      : Option<Handle<Mesh>>,
    pub cutout      : Option<Handle<Mesh>>,
    pub translucent : Option<Handle<Mesh>>,
}

pub struct ChunkMesh {
    pub opaque      : Option<Mesh>,
    pub cutout      : Option<Mesh>,
    pub translucent : Option<Mesh>,
}

impl ChunkMesh {
    #[inline]
    pub fn new(
        chunk:      Arc<Chunk>,
        accessor:  [Arc<Chunk>; 4],
        blocks:     Arc<BlockRecord>,
        mesh_cache: Arc<BlockMeshCache>,
        materials:  Arc<ArrayTexture>,
    ) -> Self {
        let mut opaque = MeshBuffer::new();
        let mut cutout = MeshBuffer::new();
        let mut translucent = MeshBuffer::new();

        chunk.sections
            .iter()
            .for_each(|(&index, section)| {
                let world_height = index.into_world_height();

                for x in 0..SECTION_SIZE {
                    for y in 0..SECTION_SIZE {
                        for z in 0..SECTION_SIZE {
                            let b_type = section.get([x, y, z]);

                            if b_type.is_air() {
                                continue;
                            }

                            let cached = mesh_cache.get_mesh(b_type)
                                .unwrap();

                            for f in 0..6u8 {
                                let facing = Facing::try_from(f).unwrap();
                            }
                        }
                    }
                }
            });

        Self {
            opaque: if opaque.len() > 0 { Some(opaque.mesh()) } else { None },
            cutout: if cutout.len() > 0 { Some(cutout.mesh()) } else { None },
            translucent: if translucent.len() > 0 { Some(translucent.mesh()) } else { None },
        }
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.opaque.is_none()
            && self.cutout.is_none()
            && self.translucent.is_none()
    }
}

pub struct ChunkMeshBuilder {
    opaque      : MeshBuffer,
    cutout      : MeshBuffer,
    translucent : MeshBuffer,
}

impl ChunkMeshBuilder {

    #[inline]
    pub fn build(self) -> ChunkMesh {
        let opaque = if self.opaque.len() > 0 { 
            Some(self.opaque.mesh())
        } else { None };
        
        let cutout = if self.cutout.len() > 0 {
            Some(self.cutout.mesh())
        } else { None };
        
        let translucent = if self.translucent.len() > 0 {
            Some(self.translucent.mesh())
        } else { None };
        
        ChunkMesh { opaque, cutout, translucent }
    }
    
    #[inline]
    fn push_quad(
        &mut self, 
        quad: Quad,
        tint: Option<[f32; 4]>,
        offset: impl Into<[f32; 3]>,
    ) {
        match quad.render_mode() {
            RenderMode::Opaque => self.opaque.push_quad_with_offset(quad, tint, offset),
            RenderMode::Cutout => self.cutout.push_quad_with_offset(quad, tint, offset),
            RenderMode::Translucent => self.translucent.push_quad_with_offset(quad, tint, offset),
        }
    }
}

#[inline(always)]
fn get_neighbor_at(
    pos: impl Into<IVec3>,
    neighbor: Neighbor,
    chunk: &Chunk,
    neighbors: &[Arc<Chunk>; 4],
) -> Option<BlockType> {
    let pos = pos.into();

    if pos.x >= 0 && pos.x < SECTION_SIZE && pos.z >= 0 && pos.z < SECTION_SIZE {
        return Some(chunk.get_at(pos));
    }

    let localized = IVec3::new(
        pos.x.rem_euclid(SECTION_SIZE),
        pos.y,
        pos.z.rem_euclid(SECTION_SIZE),
    );

    let neighbor = neighbors[neighbor as usize].as_ref();

    Some(neighbor.get_at(localized))
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
enum Neighbor {
    East = 0,
    West = 1,
    North = 2,
    South = 3,
}