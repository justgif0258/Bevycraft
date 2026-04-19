use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use bevycraft_render::prelude::{ChunkMesh, MeshBuffer};
use bevycraft_world::prelude::{Chunk, ChunkManager, ChunkPos, ChunkState};

#[derive(Component)]
pub struct ComputeChunkMesh(pub Task<ChunkMesh>);

pub fn queue_mesh_tasks(
    mut commands:   Commands,
    chunk_manager:  Res<ChunkManager>,
    query:          Query<(Entity, &ChunkPos), With<Chunk>>,
    all_chunks:     Query<&Chunk>
) {
    let thread_pool = AsyncComputeTaskPool::get();

    for (entity, pos) in &query {
        let neighbors = [
            pos.0 + IVec2::X, pos.0 - IVec2::X,
            pos.0 + IVec2::Y, pos.0 - IVec2::Y,
        ];

        let mut ready_to_mesh = true;
        let mut neighbor_data = Vec::new();

        for neighbor in neighbors {
            if let Some(&n_entity) = chunk_manager.get_chunk_entity(&neighbor) {
                if let Ok(data) = all_chunks.get(n_entity) {
                    neighbor_data.push(data);
                } else {
                    ready_to_mesh = false;
                    break;
                }
            } else {
                ready_to_mesh = false;
                break;
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