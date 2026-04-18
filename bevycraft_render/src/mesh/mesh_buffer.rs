use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::Mesh;
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
    pub fn push_quad(
        &mut self,
        quad: &Quad,
        tint: Option<[f32; 4]>,
        offset: [f32; 3],
    ) {
        let pos = quad.positions();

        let actual_pos = [
            [pos[0][0] + offset[0], pos[0][1] + offset[1], pos[0][2] + offset[2]],
            [pos[1][0] + offset[0], pos[1][1] + offset[1], pos[1][2] + offset[2]],
            [pos[2][0] + offset[0], pos[2][1] + offset[1], pos[2][2] + offset[2]],
            [pos[3][0] + offset[0], pos[3][1] + offset[1], pos[3][2] + offset[2]],
        ];

        self.positions.extend_from_slice(&actual_pos);
        self.normals.extend_from_slice(&quad.normals());
        self.uvs.extend_from_slice(&quad.uvs());
        self.textures.extend_from_slice(&quad.textures());

        let tint = if let Some(tint) = tint && quad.tintable() {
            [tint; 4]
        } else { NEUTRAL_QUAD_COLOR };

        self.colors.extend_from_slice(&tint);

        let i = self.next;

        self.indices.extend([
            i, i + 1, i + 2,
            i + 2, i + 3, i
        ]);

        self.next += 4;
    }

    #[inline]
    pub fn render(self) -> Mesh {
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD
                | RenderAssetUsages::RENDER_WORLD
        )
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals)
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs)
            .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, self.colors)
            .with_inserted_attribute(ATTRIBUTE_TEXTURE_LAYER, self.textures)
            .with_inserted_indices(Indices::U32(self.indices))
    }

    #[inline]
    pub fn push_mesh(&mut self, block_mesh: &BlockMesh, tint: Option<[f32; 4]>, position: [f32; 3]) {
        block_mesh.iter()
            .for_each(|quad| {
                self.push_quad(quad, tint, position);
            });
    }
}