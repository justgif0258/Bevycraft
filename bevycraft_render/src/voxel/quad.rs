use crate::prelude::TextureId;

#[derive(Debug, Clone)]
pub struct Quad {
    vertices    : [[f32; 3]; 2],
    uvs         : [[f32; 2]; 2],
    texture     : TextureId,
    orientation : QuadDir,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QuadDir {
    PosX,
    NegX,
    PosY,
    NegY,
    PosZ,
    NegZ,
}
