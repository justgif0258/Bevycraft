use crate::chunk::system::{
    ChunkReady, ChunkUnloaded, poll_chunk_tasks, process_unload_queue, spawn_chunk_tasks,
    update_queue,
};
use crate::prelude::{ChunkGenerator, ChunkLoaderConfig, ChunkMap, GeneratorResource};
use bevy::app::{App, Plugin};
use bevy::prelude::{FixedUpdate, IntoScheduleConfigs, States, SystemSet, in_state};

#[derive(SystemSet, Debug, Clone, Eq, PartialEq, Hash)]
pub enum ChunkSet {
    Schedule,
    Dispatch,
    Integrate,
    Cleanup,
}

pub struct ChunkPlugin<S: States> {
    pub view_distance: i32,
    pub max_concurrent: usize,
    run_in_state: S,
    generator: Option<GeneratorResource>,
}

impl<S: States> ChunkPlugin<S> {
    pub fn new(view_distance: i32, max_concurrent: usize, run_in_state: S) -> Self {
        Self {
            view_distance,
            max_concurrent,
            run_in_state,
            generator: None,
        }
    }

    pub fn with_generator<G: ChunkGenerator>(
        view_distance: i32,
        max_concurrent: usize,
        run_in_state: S,
        generator: G,
    ) -> Self {
        Self {
            view_distance,
            max_concurrent,
            run_in_state,
            generator: Some(GeneratorResource::new(generator)),
        }
    }
}

impl<S: States> Plugin for ChunkPlugin<S> {
    fn build(&self, app: &mut App) {
        app.insert_resource(ChunkMap::new(self.max_concurrent))
            .insert_resource(ChunkLoaderConfig {
                view_distance: self.view_distance,
                unload_margin: 2,
            })
            .add_message::<ChunkReady>()
            .add_message::<ChunkUnloaded>()
            .configure_sets(
                FixedUpdate,
                (
                    ChunkSet::Schedule,
                    ChunkSet::Dispatch,
                    ChunkSet::Integrate,
                    ChunkSet::Cleanup,
                )
                    .chain(),
            )
            .add_systems(
                FixedUpdate,
                (
                    update_queue.in_set(ChunkSet::Schedule),
                    spawn_chunk_tasks.in_set(ChunkSet::Dispatch),
                    poll_chunk_tasks.in_set(ChunkSet::Integrate),
                    process_unload_queue.in_set(ChunkSet::Cleanup),
                )
                    .run_if(in_state(self.run_in_state.clone())),
            );

        if let Some(generator) = &self.generator {
            app.insert_resource(generator.clone());
        }
    }
}
