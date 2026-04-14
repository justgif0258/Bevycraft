use bevy::prelude::*;
use bevycraft_app::AppState;
use bevycraft_app::systems::build_textures::build_array_texture;
use bevycraft_app::systems::models::load_block_models;
use bevycraft_app::systems::register::{init_registries};
use bevycraft_render::prelude::RModelManager;

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(RModelManager::default())
        .init_state::<AppState>()
        .insert_state(AppState::default())
        .add_systems(OnEnter(AppState::RegisteringContent), init_registries)
        .add_systems(OnEnter(AppState::BuildingRModels), load_block_models)
        .add_systems(OnEnter(AppState::BuildingTextureArrays), build_array_texture)
        .insert_resource(Time::<Fixed>::from_hz(64.0))
        .run()
}
