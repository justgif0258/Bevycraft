mod mesh;
mod model;
mod renderer;
mod textures;

pub mod prelude {
    pub use crate::{
        mesh::{
            mesh_buffer::MeshBuffer, occlusion_mask::OcclusionMask, occlusion_quad::*, quad::*,
            vertex::*,
        },
        model::r_model::*,
        textures::array_texture::*,
    };
}
