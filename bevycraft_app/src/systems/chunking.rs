use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use bevycraft_world::prelude::*;
use crate::Player;

pub fn world_level_tick(
    mut map : ResMut<SparseSpatialMap>,
    player  : Query<&Transform, With<Player>>,
) {
    let player_transform = player.single()
        .unwrap();

    let pool = AsyncComputeTaskPool::get();

    map.queue_chunks_for_generation(player_transform.translation, pool);

    map.queue_chunks_for_unloading(player_transform.translation);

    map.handle_chunk_event_queue()
}
