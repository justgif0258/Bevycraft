use crate::prelude::BlockMesh;

pub struct BlockMeshManager {
    meshes: Box<[BlockMesh]>,
}

impl BlockMeshManager {
    #[inline]
    pub fn new(meshes: Vec<BlockMesh>) -> BlockMeshManager {
        Self { meshes: meshes.into_boxed_slice() }
    }
}