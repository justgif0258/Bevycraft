use bevy::prelude::*;
use bevycraft_core::prelude::Record;
use bevycraft_render::prelude::{ArrayTexture, BlockMesh, RModelManager, RenderFlags};
use crate::AppState;
use crate::records::core_records::blocks;

pub fn solve_models(
    mut commands: Commands,
    manager         : Res<RModelManager>,
    array_texture   : Res<ArrayTexture>,
) {
    let mut meshes: Vec<BlockMesh> = Vec::new();

    blocks().iter_definitions()
        .enumerate()
        .for_each(|(i, def)| {
            let mut flags = RenderFlags::empty();

            if def.translucent() {
                flags |= RenderFlags::TRANSLUCENT;
            } else {
                flags |= RenderFlags::OPAQUE;
            }

            if def.greedy_meshable() {
                flags |= RenderFlags::GREEDY_MESHABLE;
            }

            if def.occludable() {
                flags |= RenderFlags::OCCLUDABLE;
            }

            let key = blocks().idx_to_key(i)
                .unwrap()
                .prefix("block/");

            let model = manager.get(&key);

            if let Some(m) = model.cloned() {
                let mesh = BlockMesh::new(
                    m,
                    &manager,
                    &array_texture,
                    flags
                );

                meshes.push(
                    mesh.expect("Failed to bake block mesh")
                );

                info!("Successfully baked block mesh of {}", key);
            } else {
                error!("No model was defined for {}", key);
            }
        });

    commands.remove_resource::<RModelManager>();
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