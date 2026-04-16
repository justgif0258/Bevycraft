use bevy::prelude::*;
use bevycraft_core::prelude::Record;
use bevycraft_render::prelude::{ArrayTexture, BlockMesh, RModelManager, RenderFlags};
use bevycraft_world::prelude::BlockRecord;
use crate::AppState;

pub fn solve_models(
    mut commands    : Commands,
    mut next        : ResMut<NextState<AppState>>,
    blocks_registry : Res<BlockRecord>,
    manager         : Res<RModelManager>,
    array_texture   : Res<ArrayTexture>,
) {
    let mut meshes: Vec<BlockMesh> = Vec::new();

    blocks_registry.iter_definitions()
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

            let key = blocks_registry.idx_to_key(i)
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

    next.set(AppState::InGame);
}

pub fn load_block_models(
    mut manager     : ResMut<RModelManager>,
    mut next        : ResMut<NextState<AppState>>,
    blocks_registry : Res<BlockRecord>,
) {
    blocks_registry.keys()
        .iter()
        .for_each(|&key| {
            let actual_location = key.prefix("block/");

            manager.load(actual_location);
        });

    next.set(AppState::LoadingTextures);
}