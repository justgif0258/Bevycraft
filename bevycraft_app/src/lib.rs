use bevy::prelude::{Component, Resource, States};
use bevycraft_render::prelude::ArrayTexture;
use std::sync::Arc;

pub mod systems;

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
pub enum AppState {
    #[default]
    ModelDiscovery,
    AwaitModels,
    BuildArrayTexture,
    CacheMeshes,
    Finishing,
    InGame,
}

#[derive(Component)]
pub struct Player;

#[derive(Resource)]
pub struct BevycraftClient(Arc<BevycraftClientInner>);

impl BevycraftClient {
    pub fn new(textures: ArrayTexture) -> Self {
        Self(Arc::new(BevycraftClientInner { textures }))
    }
}

struct BevycraftClientInner {
    textures: ArrayTexture,
}
