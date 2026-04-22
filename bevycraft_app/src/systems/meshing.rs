use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use bevycraft_render::prelude::{ChunkMesh};
use bevycraft_world::prelude::{Chunk, ChunkAccessor, ChunkPos, ChunkState};

#[derive(Component)]
pub struct ComputeChunkMesh(pub Task<ChunkMesh>);

pub fn queue_mesh_tasks(
    mut commands: Commands,
    accessor    : Res<ChunkAccessor>,
    query       : Query<(Entity, &ChunkPos), With<ChunkState>>,
    all_chunks  : Query<&ChunkState>,
) {
    let thread_pool = AsyncComputeTaskPool::get();

    for (entity, pos) in &query {
        let mut ready_to_mesh = true;
        let mut neighbor_data: Vec<&Chunk> = Vec::new();

        for neighbor in pos.neighbors() {
            if let Some(chunk_entity) = accessor.get_chunk_entity(&neighbor) {
                if let ChunkState::Loaded(chunk) = all_chunks.get(chunk_entity).unwrap() {
                    neighbor_data.push(chunk);
                } else {
                    ready_to_mesh = false;
                    break;
                }
            }
        }

        if ready_to_mesh {
            let task = thread_pool.spawn(async move {
                let mut mesh = ChunkMesh::new();

                mesh.build()
            });

            commands.entity(entity).insert(ComputeChunkMesh(task));
            commands.entity(entity).remove::<ChunkState>();
        }
    }
}