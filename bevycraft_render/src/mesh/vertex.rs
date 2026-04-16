use bevy::mesh::{MeshVertexAttribute, VertexFormat};
use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;
use crate::prelude::TextureId;

pub const ATTRIBUTE_TEXTURE_LAYER: MeshVertexAttribute =
    MeshVertexAttribute::new("Vertex_Layer", 1, VertexFormat::Uint32);

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct VertexMaterial {
    #[texture(0, dimension = "2d_array")]
    #[sampler(1)]
    pub array_texture: Handle<Image>
}

impl Material for VertexMaterial {
    #[inline]
    fn vertex_shader() -> ShaderRef {
        "bevycraft/shaders/texture/array_texture.wgsl".into()
    }

    #[inline]
    fn fragment_shader() -> ShaderRef {
        "bevycraft/shaders/texture/array_texture.wgsl".into()
    }

    #[inline]
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Mask(0.5)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Vertex {
    pub position: [f32; 3],
    pub uv      : [f32; 2],
    pub normal  : [f32; 3],
    pub texture : TextureId,
}