use bevy::prelude::{in_state, States};
use {
    crate::{
        prelude::ChunkEntityMap,
        renderer::system::{
            cleanup_chunk_entities, poll_mesh_tasks, remesh_dirty_chunks, trigger_chunk_meshing,
        },
    },
    bevy::{
        app::App,
        prelude::{IntoScheduleConfigs, Plugin, Res, Update},
    },
    bevycraft_world::prelude::{ChunkMap, ChunkSet},
};

pub struct ChunkRenderPlugin<S: States> {
    run_in_state: S,
}

impl<S: States> ChunkRenderPlugin<S> {
    pub fn new(run_in_state: S) -> Self {
        Self { run_in_state }
    }
}

impl<S: States> Plugin for ChunkRenderPlugin<S> {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChunkEntityMap>().add_systems(
            Update,
            (
                trigger_chunk_meshing.in_set(ChunkSet::Dispatch),
                poll_mesh_tasks.in_set(ChunkSet::Integrate),
                remesh_dirty_chunks
                    .in_set(ChunkSet::Dispatch)
                    .run_if(any_chunk_dirty),
                cleanup_chunk_entities.in_set(ChunkSet::Cleanup),
            )
                .run_if(in_state(self.run_in_state.clone())),
        );
    }
}

fn any_chunk_dirty(chunk_map: Res<ChunkMap>) -> bool {
    chunk_map.chunks.values().any(|c| c.dirty)
}
