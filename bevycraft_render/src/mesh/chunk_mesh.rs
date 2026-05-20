use bevy::prelude::Mesh;

pub struct ChunkMeshOutput {
    pub opaque: Option<Mesh>,
    pub cutout: Option<Mesh>,
    pub translucent: Option<Mesh>,
}