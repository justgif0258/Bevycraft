use bevy::prelude::States;

pub mod plugins;
pub mod systems;
pub mod records;

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
pub enum AppState {
    #[default]
    RegisteringContent,
    LoadingRModels,
    LoadingTextures,
    BuildingArrayTexture,
    SolvingBlockModels,
    BlockStateCaching,
    Finalizing,
    InGame,
}