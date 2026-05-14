use {
    crate::prelude::RenderMode,
    bevy::{
        mesh::*,
        pbr::*,
        prelude::*,
        render::render_resource::*,
        shader::{ShaderDefVal, ShaderRef},
    },
};

pub const ATTRIBUTE_TEXTURE_LAYER: MeshVertexAttribute = MeshVertexAttribute::new(
    "Vertex_Layer",
    Mesh::FIRST_AVAILABLE_CUSTOM_ATTRIBUTE,
    VertexFormat::Uint32,
);

const MATERIAL_SHADER_PATH: &'static str = "bevycraft/shaders/material.wgsl";

const PREPASS_SHADER_PATH: &'static str = "bevycraft/shaders/prepass.wgsl";

const CUTOUT_THRESHOLD: f32 = 0.5;

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
#[bind_group_data(VertexMaterialKey)]
pub struct VertexMaterial {
    #[texture(0, dimension = "2d_array")]
    #[sampler(1)]
    pub array_texture: Handle<Image>,

    pub render_mode: RenderMode,
}

impl Material for VertexMaterial {
    #[inline]
    fn vertex_shader() -> ShaderRef {
        MATERIAL_SHADER_PATH.into()
    }

    #[inline]
    fn fragment_shader() -> ShaderRef {
        MATERIAL_SHADER_PATH.into()
    }

    #[inline]
    fn alpha_mode(&self) -> AlphaMode {
        match self.render_mode {
            RenderMode::Opaque => AlphaMode::Opaque,
            RenderMode::Cutout => AlphaMode::Mask(CUTOUT_THRESHOLD),
            RenderMode::Translucent => AlphaMode::Blend,
        }
    }

    #[inline]
    fn enable_prepass() -> bool {
        true
    }

    #[inline]
    fn enable_shadows() -> bool {
        true
    }

    #[inline]
    fn prepass_vertex_shader() -> ShaderRef {
        PREPASS_SHADER_PATH.into()
    }

    #[inline]
    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let mut shader_defs: Vec<ShaderDefVal> = vec![];

        let mut vertex_attributes = vec![
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_NORMAL.at_shader_location(1),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(2),
            ATTRIBUTE_TEXTURE_LAYER.at_shader_location(8),
        ];

        if layout.0.contains(Mesh::ATTRIBUTE_TANGENT) {
            shader_defs.push("VERTEX_TANGENTS".into());

            vertex_attributes.push(Mesh::ATTRIBUTE_TANGENT.at_shader_location(4));
        }

        if layout.0.contains(Mesh::ATTRIBUTE_COLOR) {
            shader_defs.push("VERTEX_COLORS".into());

            vertex_attributes.push(Mesh::ATTRIBUTE_COLOR.at_shader_location(7));
        }

        match key.bind_group_data.render_mode {
            RenderMode::Cutout => shader_defs.push("VERTEX_CUTOUT".into()),
            _ => {}
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

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct VertexMaterialKey {
    pub render_mode: RenderMode,
}

impl From<&VertexMaterial> for VertexMaterialKey {
    fn from(value: &VertexMaterial) -> Self {
        Self {
            render_mode: value.render_mode,
        }
    }
}
