use bevy::mesh::*;
use bevy::pbr::*;
use bevy::prelude::*;
use bevy::render::render_resource::*;
use bevy::shader::{ShaderDefVal, ShaderRef};
use crate::prelude::TextureId;

pub const ATTRIBUTE_TEXTURE_LAYER: MeshVertexAttribute =
    MeshVertexAttribute::new("Vertex_Layer", Mesh::FIRST_AVAILABLE_CUSTOM_ATTRIBUTE, VertexFormat::Uint32);

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
    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>
    ) -> Result<(), SpecializedMeshPipelineError> {
        let mut shader_defs: Vec<ShaderDefVal> = vec![];

        let mut vertex_attributes = vec![
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_NORMAL.at_shader_location(1),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(2),
            ATTRIBUTE_TEXTURE_LAYER.at_shader_location(8)
        ];

        if layout.0.contains(Mesh::ATTRIBUTE_TANGENT) {
            shader_defs.push("VERTEX_TANGENTS".into());

            vertex_attributes.push(Mesh::ATTRIBUTE_TANGENT.at_shader_location(4));
        }

        if layout.0.contains(Mesh::ATTRIBUTE_COLOR) {
            shader_defs.push("VERTEX_COLORS".into());

            vertex_attributes.push(Mesh::ATTRIBUTE_COLOR.at_shader_location(7));
        }

        let vertex_layout = layout.0.get_layout(&vertex_attributes)?;

        descriptor.vertex.buffers = vec![vertex_layout];

        descriptor.vertex.shader_defs.extend(shader_defs.clone());

        if let Some(fragment) = &mut descriptor.fragment {
            fragment.shader_defs.extend(shader_defs);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Vertex {
    pub position: [f32; 3],
    pub uv      : [f32; 2],
    pub normal  : [f32; 3],
    pub texture : TextureId,
}