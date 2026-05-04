mod mesh;
mod model;
mod renderer;
mod textures;

pub mod prelude {
    pub use crate::{
        mesh::{
            mesh_buffer::MeshBuffer, occludable_quad::*, occlusion_mask::OcclusionMask, quad::*,
            vertex::*,
        },
        model::r_model::*,
        textures::array_texture::*,
    };
}
