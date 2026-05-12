use crate::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

const DEFAULT_TINT: [[f32; 4]; 4] = [[0.2, 8.0, 0.2, 1.0]; 4];

#[rustfmt::skip]
pub struct MeshBuffer {
    positions:  Vec<[f32; 3]>,
    normals:    Vec<[f32; 3]>,
    uvs:        Vec<[f32; 2]>,
    textures:   Vec<u32>,
    colors:     Vec<[f32; 4]>,
    indices:    Vec<u32>,
    next:       u32,
}

impl Into<Mesh> for MeshBuffer {
    #[inline(always)]
    fn into(self) -> Mesh {
        self.mesh()
    }
}

impl MeshBuffer {
    #[inline]
    pub fn new() -> Self {
        Self {
            positions: Vec::new(),
            normals: Vec::new(),
            uvs: Vec::new(),
            textures: Vec::new(),
            colors: Vec::new(),
            indices: Vec::new(),
            next: 0,
        }
    }

    #[inline]
    pub fn push_quads_with_offset(
        &mut self,
        quads: impl Iterator<Item = Quad>,
        offset: impl Into<[f32; 3]>,
        tint: Option<[f32; 4]>,
    ) {
        let offset = offset.into();

        quads.for_each(|quad| {
            self.push_quad_with_offset(quad, offset, tint);
        })
    }

    #[inline]
    pub fn push_quad_with_offset(
        &mut self,
        quad: Quad,
        offset: impl Into<[f32; 3]>,
        tint: Option<[f32; 4]>,
    ) {
        let offset = offset.into();
        let positions = quad.positions.map(|mut pos| {
            pos[0] += offset[0];
            pos[1] += offset[1];
            pos[2] += offset[2];

            pos
        });

        self.positions.extend_from_slice(&positions);

        self.normals
            .extend_from_slice(&[quad.normal, quad.normal, quad.normal, quad.normal]);

        self.uvs.extend_from_slice(&quad.uvs);

        self.textures.extend_from_slice(&[
            quad.texture.0,
            quad.texture.0,
            quad.texture.0,
            quad.texture.0,
        ]);

        let tint = if let Some(tint) = tint
            && quad.tintable()
        {
            [tint; 4]
        } else {
            [[1.0; 4]; 4]
        };

        self.colors.extend_from_slice(&tint);

        let i = self.next;

        self.indices.extend([i, i + 1, i + 2, i + 2, i + 3, i]);

        self.next += 4;
    }

    #[inline]
    pub fn mesh(self) -> Mesh {
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, self.colors)
        .with_inserted_attribute(ATTRIBUTE_TEXTURE_LAYER, self.textures)
        .with_inserted_indices(Indices::U32(self.indices))
    }

    #[inline]
    pub fn extract_mesh(&mut self) -> Mesh {
        let mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions.clone())
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals.clone())
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs.clone())
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, self.colors.clone())
        .with_inserted_attribute(ATTRIBUTE_TEXTURE_LAYER, self.textures.clone())
        .with_inserted_indices(Indices::U32(self.indices.clone()));

        self.clear();

        mesh
    }

    #[inline]
    pub fn clear(&mut self) {
        self.positions.clear();
        self.normals.clear();
        self.uvs.clear();
        self.colors.clear();
        self.textures.clear();
        self.indices.clear();
    }

    #[inline(always)]
    pub const fn len(&self) -> usize {
        self.indices.len()
    }
}
