use bevy::prelude::{Commands, Message, MessageWriter, Query, Transform, With};
use bevycraft_world::prelude::{Chunk, ChunkPos};
use crate::Player;

pub fn dispatch_chunk_load_requests(
    mut commands: Commands,
    mut request: MessageWriter<ChunkLoadRequest>,
    player: Query<&Transform, With<Player>>,
) {
    let player_chunk_pos = ChunkPos::from_world_pos(player.single().unwrap().translation);
}

#[derive(Message)]
pub struct ChunkLoadRequest(ChunkPos);

#[derive(Message)]
pub struct ChunkLoaded(ChunkPos, Chunk);