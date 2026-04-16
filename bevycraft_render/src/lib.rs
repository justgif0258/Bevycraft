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
            vertex::*,
            block_mesh::*,
            quad::*,
            block_mesh_manager::*,
        },
        model::{
            r_model::*,
            r_model_manager::RModelManager,
        },
    };
}
