use crate::prelude::{ChunkMeshOutput, RenderMode};
use bevy::platform::collections::HashMap;
use bevy::prelude::{Component, Entity, Resource};
use bevy::tasks::Task;
use bevycraft_world::prelude::ChunkPos;

#[derive(Component)]
pub struct ChunkMeshRoot(pub ChunkPos);

#[derive(Component)]
pub struct ChunkMeshLayer(pub RenderMode);

#[derive(Component)]
pub struct PendingMeshTask(pub Task<ChunkMeshOutput>);

#[derive(Component)]
pub struct PendingMeshBatchTask(pub Task<Vec<ChunkMeshOutput>>);

#[derive(Default, Resource)]
pub struct ChunkEntityMap(pub HashMap<ChunkPos, Entity>);