use bevy::prelude::States;

pub mod systems;

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
pub enum AppState {
    #[default]
    LoadingContent,
    WaitingForServer,
    BakingRenderers,
    InGame,
}