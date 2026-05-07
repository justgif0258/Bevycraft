mod mesh;
mod model;
mod renderer;
mod textures;

pub mod prelude {
    pub use crate::{
        mesh::{mesh_buffer::MeshBuffer, occlusion_mask::OcclusionMask, quad::*},
        model::{block_model::BlockModel, r_model::*},
        textures::{
            array_texture::{ArrayTexture, TextureId},
            material::{ATTRIBUTE_TEXTURE_LAYER, VertexMaterial},
        },
    };
}
