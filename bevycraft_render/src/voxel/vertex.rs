use crate::prelude::TextureId;

#[derive(Debug, Clone, PartialEq)]
pub struct Vertex {
    pub(crate) position : [f32; 3],
    pub(crate) uv       : [f32; 2],
    pub(crate) normal   : [f32; 3],
    pub(crate) texture  : TextureId,
}