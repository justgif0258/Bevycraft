use std::sync::Arc;
use bevy::prelude::{IVec3, Mesh};
use bevycraft_world::prelude::*;
use crate::prelude::*;

pub struct ChunkMesh {
    pub opaque      : Option<Mesh>,
    pub cutout      : Option<Mesh>,
    pub translucent : Option<Mesh>,
}

impl ChunkMesh {
    #[inline]
    pub fn new(
        chunk:      &Chunk,
        chunk_pos:  &ChunkPos,
        level:      &Level,
        mesh_cache: Arc<BlockMeshCache>,
    ) -> Self {
        let mut builder = ChunkMeshBuilder::new();
        
        let world_pos = chunk_pos.into_world_pos().as_ivec3();

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
                                
                                let neighbor_pos = match facing {
                                    Facing::PosX => IVec3::new(world_pos.x + 1, world_pos.y, world_pos.z),
                                    Facing::NegX => IVec3::new(world_pos.x - 1, world_pos.y, world_pos.z),
                                    Facing::PosY => IVec3::new(world_pos.x, world_pos.y + 1, world_pos.z),
                                    Facing::NegY => IVec3::new(world_pos.x, world_pos.y - 1, world_pos.z),
                                    Facing::PosZ => IVec3::new(world_pos.x, world_pos.y, world_pos.z + 1),
                                    Facing::NegZ => IVec3::new(world_pos.x, world_pos.y, world_pos.z - 1),
                                };
                                
                                if let Some(neighbor) = level.get_at(neighbor_pos)
                                    && let Some(mask) = mesh_cache.get_occlusion_mask(neighbor, !facing)
                                {
                                    if cached.is_occluded_at(facing, mask) { 
                                        continue;
                                    }
                                    
                                    builder.push_quads(
                                        cached.get_occlusion_quads_at(facing),
                                        Some([0.2, 0.8, 0.2, 1.0]),
                                        [x as f32, (world_height + y) as f32, z as f32],
                                    )
                                }
                            }
                            
                            builder.push_quads(
                                cached.get_inner_quads(),
                                Some([0.2, 0.8, 0.2, 1.0]),
                                [x as f32, (world_height + y) as f32, z as f32],
                            )
                        }
                    }
                }
            });

        builder.build()
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.opaque.is_none()
            && self.cutout.is_none()
            && self.translucent.is_none()
    }
}

struct ChunkMeshBuilder {
    opaque      : MeshBuffer,
    cutout      : MeshBuffer,
    translucent : MeshBuffer,
}

impl ChunkMeshBuilder {
    #[inline]
    fn new() -> Self {
        Self {
            opaque: MeshBuffer::new(),
            cutout: MeshBuffer::new(),
            translucent: MeshBuffer::new(),
        }
    }

    #[inline]
    fn build(self) -> ChunkMesh {
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
    fn push_quads(
        &mut self,
        quads: &[Quad],
        tint: Option<[f32; 4]>,
        offset: impl Into<[f32; 3]>,
    ) {
        let offset = offset.into();
        
        quads.iter()
            .for_each(|&quad| self.push_quad(quad, tint, offset))
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