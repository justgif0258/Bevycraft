use bevy::prelude::Resource;
use crate::prelude::{ArrayTexture, BlockMesh, RModel, RModelManager, RenderFlags};

#[derive(Resource)]
pub struct BlockMeshManager {
    meshes: Box<[BlockMesh]>,
}

impl BlockMeshManager {
    #[inline]
    pub const fn builder() -> BlockMeshManagerBuilder {
        BlockMeshManagerBuilder { meshes: Vec::new() }
    }

    #[inline(always)]
    pub fn get_mesh(&self, mesh_index: u32) -> Option<&BlockMesh> {
        self.meshes.get(mesh_index as usize)
    }
}

#[derive(Default)]
pub struct BlockMeshManagerBuilder {
    meshes: Vec<BlockMesh>,
}

impl BlockMeshManagerBuilder {
    #[inline]
    pub fn add_mesh(&mut self, mesh: BlockMesh) {
        self.meshes.push(mesh);
    }

    #[inline]
    pub fn bake_and_add_mesh(
        &mut self,
        manager         : &RModelManager,
        array_texture   : &ArrayTexture,
        model           : RModel,
        render_flags    : RenderFlags
    ) -> Result<(), Vec<String>> {
        let mesh = BlockMesh::new(
            model,
            &manager,
            &array_texture,
            render_flags,
        );

        match mesh {
            Ok(mesh) => {
                self.meshes.push(mesh);

                Ok(())
            },
            Err(errors) => Err(errors),
        }
    }

    #[inline]
    pub fn build(self) -> BlockMeshManager {
        BlockMeshManager { meshes: self.meshes.into_boxed_slice() }
    }
}