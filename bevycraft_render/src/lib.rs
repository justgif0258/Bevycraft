mod voxel;
mod textures;
pub mod mesh;

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
        }
    };
}
