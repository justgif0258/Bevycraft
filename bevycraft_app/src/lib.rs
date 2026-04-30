use bevy::prelude::{Component, Resource, States};
use bevycraft_render::prelude::{ArrayTexture, BlockMeshCache, RModelManager};
use bevycraft_world::prelude::BlockRecord;
use std::sync::Arc;

pub mod systems;

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
pub enum AppState {
    #[default]
    LoadingContent,
    LockingRegistries,
    WaitingForServer,
    MeshCaching,
    InGame,
}

#[derive(Component)]
pub struct Player;

#[derive(Resource)]
pub struct BlockRenderer {
    pub meshes: Arc<BlockMeshCache>,
    pub materials: Arc<ArrayTexture>,
}

impl Clone for BlockRenderer {
    #[inline(always)]
    fn clone(&self) -> Self {
        Self {
            meshes: self.meshes.clone(),
            materials: self.materials.clone(),
        }
    }
}

#[derive(Resource)]
pub struct GlobalRecords {
    pub blocks: BlockRecord,
}

impl Clone for GlobalRecords {
    #[inline(always)]
    fn clone(&self) -> Self {
        Self {
            blocks: self.blocks.clone(),
        }
    }
}

#[derive(Resource)]
pub struct BevycraftClient(Arc<BevycraftClientInner>);

impl BevycraftClient {
    pub fn new(textures: ArrayTexture, meshes: BlockMeshCache) -> Self {
        Self(Arc::new(BevycraftClientInner { textures, meshes }))
    }
}

struct BevycraftClientInner {
    textures: ArrayTexture,
    meshes: BlockMeshCache,
}
