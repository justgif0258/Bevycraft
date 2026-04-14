use bevy::prelude::*;
use bevycraft_core::prelude::Record;
use bevycraft_render::prelude::RModelManager;
use crate::records::core_records::blocks;

pub fn load_block_models(
    mut manager: ResMut<RModelManager>,
) {
    blocks().keys()
        .iter()
        .for_each(|&key| {
            let actual_location = key.prefix("block/");

            manager.load(actual_location);
        });
}