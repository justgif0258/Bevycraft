use crate::chunk::systems::{
    ChunkReadyEvent, ChunkUnloadedEvent, poll_chunk_tasks, process_unload_queue, spawn_chunk_tasks,
    update_load_queue,
};
use crate::prelude::{ChunkGenerator, ChunkLoaderConfig, ChunkMap, GeneratorResource};
use bevy::app::{App, Plugin};
use bevy::prelude::{FixedUpdate, IntoScheduleConfigs, SystemSet};

#[derive(SystemSet, Debug, Clone, Eq, PartialEq, Hash)]
pub enum ChunkSet {
    Schedule,
    Dispatch,
    Integrate,
    Cleanup,
}

pub struct ChunkPlugin {
    pub view_distance: i32,
    pub max_concurrent: usize,
    generator: Option<GeneratorResource>,
}

impl ChunkPlugin {
    pub fn new(view_distance: i32, max_concurrent: usize) -> Self {
        Self {
            view_distance,
            max_concurrent,
            generator: None,
        }
    }

    pub fn with_generator<G: ChunkGenerator>(
        view_distance: i32,
        max_concurrent: usize,
        generator: G,
    ) -> Self {
        Self {
            view_distance,
            max_concurrent,
            generator: Some(GeneratorResource::new(generator)),
        }
    }
}

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ChunkMap::new(self.max_concurrent))
            .insert_resource(ChunkLoaderConfig {
                view_distance: self.view_distance,
                unload_margin: 2,
            })
            .add_message::<ChunkReadyEvent>()
            .add_message::<ChunkUnloadedEvent>()
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
                    update_load_queue.in_set(ChunkSet::Schedule),
                    spawn_chunk_tasks.in_set(ChunkSet::Dispatch),
                    poll_chunk_tasks.in_set(ChunkSet::Integrate),
                    process_unload_queue.in_set(ChunkSet::Cleanup),
                ),
            );

        if let Some(generator) = &self.generator {
            app.insert_resource(generator.clone());
        }
    }
}
