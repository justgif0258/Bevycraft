mod mesh;
mod model;
mod renderer;
mod textures;

pub mod prelude {
    pub use crate::{
        mesh::{buffer::VertexBuffer, chunk_mesh::*, occlusion_mask::OcclusionMask, quad::*},
        model::{block_model::BlockModel, cache::*, manager::ModelManager, r_model::*, Model},
        textures::{
            array_texture::{ArrayTexture, TextureId, NULL_TEXTURE_ID, NULL_TEXTURE_LOCATION},
            material::{VertexMaterial, ATTRIBUTE_TEXTURE_LAYER},
            texture_manager::{TextureBakery, TextureManager},
        },
    };
}
