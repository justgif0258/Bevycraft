mod mesh;
mod model;
mod renderer;
mod textures;

pub mod prelude {
    pub use crate::{
        mesh::{buffer::*, chunk::*, input::MeshInput, mask::OcclusionMask, quad::*},
        model::{block_model::BlockModel, cache::*, manager::ModelManager, r_model::*, Model},
        renderer::{component::*, plugin::ChunkRenderPlugin},
        textures::{
            array_texture::{ArrayTexture, TextureId, NULL_TEXTURE_ID, NULL_TEXTURE_LOCATION},
            material::{VertexMaterial, ATTRIBUTE_TEXTURE_LAYER},
            texture_manager::{TextureBakery, TextureManager},
        },
    };
}
