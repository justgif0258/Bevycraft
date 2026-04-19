use std::mem::transmute;
use bevy::prelude::*;
use bevy::tasks::futures_lite::future;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use bevycraft_world::prelude::*;
use crate::{GlobalRecords, Player};

#[derive(Component)]
pub struct ComputeChunkData(pub Task<Chunk>);

pub fn manage_chunks(
    mut commands: Commands,
    mut chunks  : ResMut<ChunkManager>,
    global      : Res<GlobalRecords>,
    player_query: Query<&Transform, With<Player>>
) {
    let Ok(player_transform) = player_query.single() else { return; };

    let player_pos = player_transform.translation;
    let current_chunk_x = (player_pos.x / 16.0).floor() as i32;
    let current_chunk_z = (player_pos.z / 16.0).floor() as i32;
    let player_chunk_pos = IVec2::new(current_chunk_x, current_chunk_z);

    let view_distance = chunks.view_distance as i32;

    let thread_pool = AsyncComputeTaskPool::get();

    for x in -view_distance..view_distance {
        for z in -view_distance..view_distance {
            let target_chunk_pos = IVec2::new(
                player_chunk_pos.x + x,
                player_chunk_pos.y + z,
            );

            if !chunks.is_loaded(&target_chunk_pos) {
                let generator_clone = chunks.get_generator();
                let blocks_clone = global.blocks.clone();

                let task = thread_pool.spawn(async move {
                    let mut chunk_data = Chunk::default();

                    generator_clone.generate_chunk(target_chunk_pos, &mut chunk_data, blocks_clone);

                    chunk_data
                });

                let chunk_entity = commands.spawn((
                    ChunkPos(target_chunk_pos),
                    ComputeChunkData(task),
                    ChunkState::WaitingForData
                )).id();

                chunks.insert_chunk_entity(target_chunk_pos, chunk_entity);
            }
        }
    }

    chunks.active.retain(|&hash, &mut entity| {
        let chunk_pos: IVec2 = unsafe { transmute(hash) };

        let dist_x = (chunk_pos.x - player_chunk_pos.x).abs();
        let dist_z = (chunk_pos.y - player_chunk_pos.y).abs();

        if dist_x > view_distance || dist_z > view_distance {
            commands.entity(entity).despawn();

            return false;
        }

        true
    });
}

pub fn handle_chunk_tasks(
    mut commands: Commands,
    mut tasks_query: Query<(Entity, &mut ComputeChunkData)>
) {
    for (entity, mut task) in &mut tasks_query {
        if let Some(chunk_data) = future::block_on(future::poll_once(&mut task.0)) {
            commands.entity(entity).insert(chunk_data);

            commands.entity(entity).remove::<ComputeChunkData>();

            commands.entity(entity).insert(ChunkState::WaitingForMesh);
        }
    }
}