use std::sync::Arc;
use bevy::prelude::{Component, Resource, States};
use bevycraft_render::prelude::{ArrayTexture, BlockMeshCache};
use bevycraft_world::prelude::BlockRecord;

pub mod systems;

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
pub enum AppState {
    #[default]
    LoadingContent,
    WaitingForServer,
    BakingRenderers,
    InGame,
}

#[derive(Component)]
pub struct Player;

#[derive(Resource)]
pub struct BlockRenderer {
    pub meshes: Arc<BlockMeshCache>,
    pub materials: Arc<ArrayTexture>
}

impl Clone for BlockRenderer {
    #[inline(always)]
    fn clone(&self) -> Self {
        Self {
            meshes: self.meshes.clone(),
            materials: self.materials.clone()
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
