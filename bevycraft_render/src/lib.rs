mod mesh;
mod model;
mod renderer;
mod textures;

pub mod prelude {
    pub use crate::{
        mesh::{block_mesh::*, mesh_buffer::MeshBuffer, quad::*, vertex::*},
        model::r_model::*,
        textures::array_texture::*,
    };
}
