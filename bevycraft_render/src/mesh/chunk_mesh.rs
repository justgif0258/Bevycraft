use bevy::prelude::{Handle, Mesh};
use crate::prelude::{MeshBuffer, Quad, RenderMode};

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
    pub fn new() -> ChunkMeshBuilder {
        ChunkMeshBuilder {
            opaque: MeshBuffer::new(),
            cutout: MeshBuffer::new(),
            translucent: MeshBuffer::new(),
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
    pub fn push_quad(
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