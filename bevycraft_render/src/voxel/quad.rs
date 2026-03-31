#[derive(Debug, Clone)]
#[rustfmt::skip]
pub struct Quad {
    vertices    : [[f32; 3]; 4],
    uvs         : [[f32; 2]; 4],
    texture     : usize,
    orientation : Orientation,
}

#[derive(Debug, Clone, Copy)]
pub enum Orientation {
    PosX,
    NegX,
    PosY,
    NegY,
    PosZ,
    NegZ,
}
