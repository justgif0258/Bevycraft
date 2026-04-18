use bevy::platform::collections::HashMap;
use bevy::platform::hash::NoOpHash;
use bevy::prelude::Resource;
use frozen_collections::{FzHashMap};
use crate::prelude::{ArrayTexture, BlockMesh, Facing, OcclusionMask, RModel, RModelManager, RenderFlags};

#[derive(Resource)]
pub struct BlockMeshManager {
    meshes: FzHashMap<u64, BlockMesh, NoOpHash>,
}

impl BlockMeshManager {
    #[inline]
    pub const fn builder() -> BlockMeshManagerBuilder {
        BlockMeshManagerBuilder { entries: HashMap::with_hasher(NoOpHash) }
    }

    #[inline(always)]
    pub fn get_mesh(&self, block_index: u32) -> Option<&BlockMesh> {
        self.meshes.get(&(block_index as u64))
    }
    
    #[inline(always)]
    pub fn get_occlusion_mask(&self, block_index: u32, facing: Facing) -> Option<OcclusionMask> {
        let mesh = self.get_mesh(block_index)?;
        
        Some(mesh.occlusion_mask(facing))
    }
}

#[derive(Default)]
pub struct BlockMeshManagerBuilder {
    entries: HashMap<u64, BlockMesh, NoOpHash>
}

impl BlockMeshManagerBuilder {
    #[inline]
    pub fn add_mesh(&mut self, mesh: BlockMesh, block_id: u32) {
        self.entries.insert(block_id as u64, mesh);
    }

    #[inline]
    pub fn bake_and_add_mesh(
        &mut self,
        manager         : &RModelManager,
        array_texture   : &ArrayTexture,
        model           : RModel,
        render_flags    : RenderFlags,
        block_id        : u32
    ) -> Result<(), Vec<String>> {
        let mesh = BlockMesh::new(
            model,
            &manager,
            &array_texture,
            render_flags,
        );

        match mesh {
            Ok(mesh) => {
                self.entries.insert(block_id as u64, mesh);

                Ok(())
            },
            Err(errors) => Err(errors),
        }
    }

    #[inline]
    pub fn build(self) -> BlockMeshManager {
        let meshes: FzHashMap<u64, BlockMesh, NoOpHash> = FzHashMap::from_iter(
            self.entries.into_iter()
        );

        BlockMeshManager { meshes }
    }
}