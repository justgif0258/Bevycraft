mod voxel;
mod textures;
mod mesh;
mod model;

pub mod prelude {
    pub use crate::{
        voxel::{
            quad::*,
        },
        textures::{
            texture_id::*,
            array_texture::ArrayTexture
        },
        mesh::{
            vertex::Vertex,
        },
        model::{
            r_model::*,
            r_model_manager::RModelManager,
        },
    };
}
