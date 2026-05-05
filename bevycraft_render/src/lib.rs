mod mesh;
mod model;
mod renderer;
mod textures;

pub mod prelude {
    pub use crate::{
        mesh::{mesh_buffer::MeshBuffer, occlusion_mask::OcclusionMask, quad::*, vertex::*},
        model::r_model::RModel,
        textures::array_texture::{ArrayTexture, TextureId},
    };
}
