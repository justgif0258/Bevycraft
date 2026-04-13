mod voxel;
mod textures;

pub mod prelude {
    pub use crate::{
        voxel::{
            quad::*,
            vertex::Vertex,
        },
        textures::{
            texture_id::*,
            array_texture::ArrayTexture
        }
    };
}
