use bevy::prelude::States;

pub mod plugins;
pub mod systems;

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
pub enum AppState {
    #[default]
    RegisteringContent,
    LoadingRModels,
    LoadingTextures,
    BuildingArrayTexture,
    SolvingBlockModels,
    InGame,
}