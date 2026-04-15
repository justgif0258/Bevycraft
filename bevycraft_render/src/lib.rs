mod textures;
mod mesh;
mod model;

pub mod prelude {
    pub use crate::{
        textures::{
            texture_id::*,
            array_texture::ArrayTexture
        },
        mesh::{
            vertex::Vertex,
            block_mesh::*,
            quad::*,
        },
        model::{
            r_model::*,
            r_model_manager::RModelManager,
        },
    };
}
