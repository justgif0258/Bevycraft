use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use bevy::tasks::futures::check_ready;
use bevycraft_world::prelude::*;
use bevycraft_world::prelude::ChunkState;
use crate::{GlobalRecords, Player};

#[derive(Component)]
pub struct ComputeChunkData(pub Task<Chunk>);

pub fn pool_chunks(
    mut commands: Commands,
    mut accessor: ResMut<ChunkAccessor>,
    global      : Res<GlobalRecords>,
    player_query: Query<&Transform, With<Player>>
) {
    let Ok(player_transform) = player_query.single() else { return; };

    let player_pos = player_transform.translation;
    let current_chunk_x = (player_pos.x / 16.0).floor() as i32;
    let current_chunk_z = (player_pos.z / 16.0).floor() as i32;
    let player_chunk_pos = IVec2::new(current_chunk_x, current_chunk_z);

    let view_distance = accessor.view_distance;

    let thread_pool = AsyncComputeTaskPool::get();

    for x in -view_distance..view_distance {
        for z in -view_distance..view_distance {
            let target_chunk_pos = IVec2::new(
                player_chunk_pos.x + x,
                player_chunk_pos.y + z,
            );

            if !accessor.is_loaded(&target_chunk_pos) {
                let generator_clone = accessor.get_generator();
                let blocks_clone = global.blocks.clone();

                let task = thread_pool.spawn(async move {
                    let mut chunk_data = Chunk::default();

                    generator_clone.generate_chunk(target_chunk_pos, &mut chunk_data, blocks_clone);

                    chunk_data
                });

                let chunk_entity = commands.spawn((
                    ChunkPos(target_chunk_pos),
                    ChunkState::Generating(task)
                )).id();

                accessor.insert_chunk_entity(target_chunk_pos, chunk_entity);
            }
        }
    }
}

pub fn handle_chunk_tasks(
    mut tasks_query: Query<&mut ChunkState>
) {
    for mut state in &mut tasks_query {
        let ChunkState::Generating(task) = &mut *state else { continue };

        if let Some(chunk_data) = check_ready(task) {
            *state = ChunkState::Loaded(chunk_data);
        }
    }
}

pub fn trash_chunks(
    world : &mut World,
) {
    let transform = world
        .query_filtered::<&Transform, With<Player>>()
        .single(world)
        .unwrap();

    let player_pos = transform.translation;
    let current_chunk_x = (player_pos.x / 16.0).floor() as i32;
    let current_chunk_z = (player_pos.z / 16.0).floor() as i32;
    let player_chunk_pos = IVec2::new(current_chunk_x, current_chunk_z);

    world.resource_scope(|mut world, mut accessor: Mut<ChunkAccessor>| {
        let mut queue = accessor.unload_chunks_out_of_range(player_chunk_pos, &mut world);
        queue.apply(&mut world);
    });
}
