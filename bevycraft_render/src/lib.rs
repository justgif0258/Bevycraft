mod textures;
mod mesh;
mod model;
pub mod renderer;

pub mod prelude {
    pub use crate::{
        textures::{
            texture_id::*,
            array_texture::ArrayTexture
        },
        mesh::{
            vertex::*,
            block_mesh::*,
            quad::*,
            mesh_buffer::MeshBuffer,
            chunk_mesh::*,
        },
        model::{
            r_model::*,
            r_model_manager::RModelManager,
        },
    };
}
