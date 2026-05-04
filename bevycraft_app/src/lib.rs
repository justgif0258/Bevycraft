use bevy::prelude::{Component, Resource, States};
use bevycraft_render::prelude::ArrayTexture;
use std::sync::Arc;

pub mod systems;

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
pub enum AppState {
    #[default]
    ModelDiscovery,
    FinishingLoadingModels,
    TextureDiscovery,
    CachingMeshes,
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
