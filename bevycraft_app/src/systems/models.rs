use bevy::prelude::*;
use bevycraft_core::prelude::Record;
use bevycraft_render::prelude::RModelManager;
use crate::AppState;
use crate::records::core_records::blocks;

pub fn solve_models(
    manager: Res<RModelManager>,
) {
    
}

pub fn load_block_models(
    mut manager: ResMut<RModelManager>,
    mut next: ResMut<NextState<AppState>>,
) {
    blocks().keys()
        .iter()
        .for_each(|&key| {
            let actual_location = key.prefix("block/");

            manager.load(actual_location);
        });

    next.set(AppState::LoadingTextures);
}