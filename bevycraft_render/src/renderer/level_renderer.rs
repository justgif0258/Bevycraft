use std::cell::RefCell;
use std::sync::Arc;
use bevy::platform::collections::HashMap;
use bevy::platform::hash::NoOpHash;
use bevy::prelude::*;
use crossbeam::channel::{unbounded, Receiver, Sender};
use bevycraft_world::prelude::*;
use crate::prelude::*;

thread_local! {
    static LOCAL_BUFFER: RefCell<MeshBuffer> = RefCell::new(MeshBuffer::new());
}

#[derive(Resource)]
pub struct LevelRenderer {
    active_meshes: HashMap<ChunkPos, Entity, NoOpHash>,

    mesh_rx: Receiver<(ChunkPos, Mesh)>,
    mesh_tx: Sender<(ChunkPos, Mesh)>,

    mesh_cache: Arc<BlockMeshCache>,
    materials: Arc<ArrayTexture>,
}

impl LevelRenderer {
    #[inline]
    pub fn new(
        mesh_cache: Arc<BlockMeshCache>,
        materials: Arc<ArrayTexture>,
    ) -> Self {
        let (mesh_tx, mesh_rx) = unbounded();

        Self {
            active_meshes: HashMap::with_hasher(NoOpHash),

            mesh_rx,
            mesh_tx,

            mesh_cache,
            materials
        }
    }

    #[inline]
    pub fn remove_chunk_mesh(
        &mut self,
        commands: &mut Commands,
        chunk_pos: &ChunkPos,
    ) {
        if let Some(entity) = self.active_meshes.remove(chunk_pos) {
            commands.entity(entity)
                .despawn();
        }
    }

    #[inline]
    pub fn is_chunk_rendered(&self, chunk_pos: ChunkPos) -> bool {
        self.active_meshes.contains_key(&chunk_pos)
    }
}

#[derive(Component)]
pub struct ChunkMeshEntity;