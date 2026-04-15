use bevy::prelude::*;
use bevycraft_app::AppState;
use bevycraft_app::systems::build_textures::{build_array_texture, load_textures_into_server, wait_for_server};
use bevycraft_app::systems::models::{load_block_models, solve_models};
use bevycraft_app::systems::register::{init_registries};
use bevycraft_render::prelude::{RModelManager};

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Time::<Fixed>::from_hz(64.0))
        .insert_resource(RModelManager::default())
        .init_state::<AppState>()
        .insert_state(AppState::default())
        .add_systems(OnEnter(AppState::RegisteringContent), init_registries)
        .add_systems(OnEnter(AppState::LoadingRModels), load_block_models)
        .add_systems(OnEnter(AppState::LoadingTextures), load_textures_into_server)
        .add_systems(Update, wait_for_server
            .run_if(in_state(AppState::LoadingTextures))
        )
        .add_systems(OnEnter(AppState::BuildingArrayTexture), build_array_texture)
        .add_systems(OnEnter(AppState::SolvingBlockModels), solve_models)
        .run()
}
