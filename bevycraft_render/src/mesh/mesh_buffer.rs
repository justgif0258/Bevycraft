use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::{info, Mesh};
use crate::prelude::*;

const NEUTRAL_QUAD_COLOR: [[f32; 4]; 4] = [NEUTRAL_TINT; 4];

pub struct MeshBuffer {
    positions   : Vec<[f32; 3]>,
    normals     : Vec<[f32; 3]>,
    uvs         : Vec<[f32; 2]>,
    colors      : Vec<[f32; 4]>,
    textures    : Vec<u32>,
    indices     : Vec<u32>,
    next        : u32,
}

impl MeshBuffer {
    #[inline]
    pub fn with_expected_capacity(capacity: usize) -> Self {
        Self {
            positions: Vec::with_capacity(capacity),
            normals: Vec::with_capacity(capacity),
            uvs: Vec::with_capacity(capacity),
            colors: Vec::with_capacity(capacity),
            textures: Vec::with_capacity(capacity),
            indices: Vec::with_capacity(capacity),
            next: 0,
        }
    }

    #[inline]
    pub fn push_quad(&mut self, quad: &Quad, tint: Option<[f32; 4]>) {
        self.positions.extend_from_slice(&quad.positions());
        self.normals.extend_from_slice(&quad.normals());
        self.uvs.extend_from_slice(&quad.uvs());
        self.textures.extend_from_slice(&quad.textures());

        let tint = if let Some(tint) = tint && quad.tintable() {
            [tint; 4]
        } else { NEUTRAL_QUAD_COLOR };

        self.colors.extend_from_slice(&tint);

        self.next += 4;
    }

    #[inline]
    pub fn render(self) -> Mesh {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD
                | RenderAssetUsages::RENDER_WORLD
        );

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, self.colors);
        mesh.insert_attribute(ATTRIBUTE_TEXTURE_LAYER, self.textures);

        mesh.insert_indices(Indices::U32(self.indices));

        mesh
    }

    #[cfg(debug_assertions)]
    #[inline]
    pub fn push_mesh(&mut self, block_mesh: &BlockMesh, tint: Option<[f32; 4]>) {
        block_mesh.iter_quads()
            .for_each(|quad| {
                info!("{} quad: {:?}", quad.facing(), quad);

                self.push_quad(quad, tint);
            });
    }
}