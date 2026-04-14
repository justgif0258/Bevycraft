use crate::prelude::TextureId;

#[derive(Debug, Clone, PartialEq)]
pub struct Vertex {
    pub position: [f32; 3],
    pub uv      : [f32; 2],
    pub normal  : [f32; 3],
    pub texture : TextureId,
}