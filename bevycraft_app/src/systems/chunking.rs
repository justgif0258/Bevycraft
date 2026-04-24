use bevy::prelude::{Commands, Query, Res, ResMut, Transform, With};
use bevy::tasks::AsyncComputeTaskPool;
use bevycraft_world::prelude::Level;
use crate::Player;

pub fn world_level_tick(
    mut commands: Commands,
    mut level   : ResMut<Level>,
    player      : Query<&Transform, With<Player>>,
) {
    let player_transform = player.single()
        .unwrap();

    let pool = AsyncComputeTaskPool::get();

    level.queue_chunks_for_loading(player_transform.translation, pool);

    level.queue_chunks_for_unloading(player_transform.translation);
}