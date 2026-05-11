mod mesh;
mod model;
mod renderer;
mod textures;

pub mod prelude {
    pub use crate::{
        mesh::{mesh_buffer::MeshBuffer, occlusion_mask::OcclusionMask, quad::*},
        model::{Model, block_model::BlockModel, model_cache::ModelCache, r_model::*},
        textures::{
            array_texture::{ArrayTexture, NULL_TEXTURE_ID, NULL_TEXTURE_LOCATION, TextureId},
            material::{ATTRIBUTE_TEXTURE_LAYER, VertexMaterial},
            texture_manager::{TextureBakery, TextureManager},
        },
    };
}
