use {
    crate::prelude::{
        mesh_chunk, ArrayTexture, BlockModel, ChunkEntityMap, ChunkMeshLayer, ChunkMeshRoot,
        Direction, MeshInput, ModelCache, PendingMeshTask, RenderMode,
    },
    bevy::{
        platform::collections::HashSet,
        prelude::{
            Assets, Children, Commands, Entity, Mesh, Mesh3d, MeshMaterial3d, MessageReader, Query,
            Res, ResMut, Transform, Visibility,
        },
        tasks::{futures::check_ready, AsyncComputeTaskPool},
    },
    bevycraft_core::prelude::Block,
    bevycraft_world::prelude::{ChunkMap, ChunkPos, ChunkReady, ChunkUnloaded},
    std::sync::LazyLock,
};

#[allow(unused)]
static PARALLELISM: LazyLock<usize> = LazyLock::new(|| AsyncComputeTaskPool::get().thread_num());

pub fn trigger_chunk_meshing(
    mut commands: Commands,
    mut events: MessageReader<ChunkReady>,
    mut entity_map: ResMut<ChunkEntityMap>,
    model_cache: Res<ModelCache<Block, BlockModel>>,
    chunk_map: Res<ChunkMap>,
) {
    let pool = AsyncComputeTaskPool::get();

    let mut positions: HashSet<ChunkPos> = HashSet::new();

    for &ChunkReady(pos) in events.read() {
        positions.insert(pos);
        for dir in Direction::ALL {
            let nb = ChunkPos::from(pos + dir.offset());

            if chunk_map.is_loaded(&nb) {
                positions.insert(nb);
            }
        }
    }

    /*
    let to_mesh = positions.into_iter().collect::<Vec<_>>();

    let total_chunks = to_mesh.len();
    let batch_size = (total_chunks + *PARALLELISM - 1) / *PARALLELISM;

    for batch in to_mesh.chunks(batch_size) {
        let mut inputs = Vec::new();

        for pos in batch {
            if let Some(chunk) = chunk_map.get(pos) {
                inputs.push(MeshInput::build(
                    *pos,
                    chunk.storage.clone(),
                    &chunk_map,
                    model_cache.clone(),
                ))
            }
        }

        let batch_task = pool.spawn(async move {
            inputs.into_iter().map(|input| mesh_chunk(input)).collect::<Vec<_>>()
        });
    }
    */

    for pos in positions {
        let Some(chunk) = chunk_map.get(&pos) else {
            continue;
        };

        if chunk.storage.is_empty() {
            continue;
        };

        let input = MeshInput::build(pos, chunk.storage.clone(), &chunk_map, model_cache.clone());

        let task = pool.spawn(async move { mesh_chunk(input) });

        match entity_map.0.get(&pos) {
            None => {
                let root = commands
                    .spawn((
                        ChunkMeshRoot(pos),
                        PendingMeshTask(task),
                        Transform::from_translation(pos.into_world_pos()),
                        Visibility::default(),
                    ))
                    .id();

                entity_map.0.insert(pos, root);
            }
            Some(&root) => {
                commands.entity(root).insert(PendingMeshTask(task));
            }
        }
    }
}

pub fn poll_mesh_tasks(
    mut commands: Commands,
    mut tasks: Query<(Entity, &ChunkMeshRoot, &mut PendingMeshTask)>,
    mut meshes: ResMut<Assets<Mesh>>,
    textures: Res<ArrayTexture>,
) {
    for (root_entity, _root, mut pending) in &mut tasks {
        let Some(output) = check_ready(&mut pending.0) else {
            continue;
        };

        commands
            .entity(root_entity)
            .remove::<PendingMeshTask>()
            .despawn_related::<Children>();

        for (mesh_opt, mode) in [
            (output.opaque, RenderMode::Opaque),
            (output.cutout, RenderMode::Cutout),
            (output.translucent, RenderMode::Translucent),
        ] {
            let Some(mesh) = mesh_opt else { continue };

            let child = commands
                .spawn((
                    ChunkMeshLayer(mode),
                    Mesh3d(meshes.add(mesh)),
                    MeshMaterial3d(textures.get_vertex_material(mode)),
                    Transform::default(),
                    Visibility::default(),
                ))
                .id();

            commands.entity(root_entity).add_child(child);
        }
    }
}

pub fn cleanup_chunk_entities(
    mut commands: Commands,
    mut events: MessageReader<ChunkUnloaded>,
    mut entity_map: ResMut<ChunkEntityMap>,
) {
    for &ChunkUnloaded(pos) in events.read() {
        if let Some(root) = entity_map.0.remove(&pos) {
            commands.entity(root).despawn();
        }
    }
}

pub fn remesh_dirty_chunks(
    mut commands: Commands,
    mut entity_map: ResMut<ChunkEntityMap>,
    chunk_map: Res<ChunkMap>,
    model_cache: Res<ModelCache<Block, BlockModel>>,
) {
    let pool = AsyncComputeTaskPool::get();

    let dirty: Vec<ChunkPos> = chunk_map
        .chunks
        .iter()
        .filter(|(_, c)| c.dirty)
        .map(|(&pos, _)| pos)
        .collect();

    if dirty.is_empty() {
        return;
    }

    let mut to_mesh: HashSet<ChunkPos> = HashSet::new();
    for pos in dirty {
        to_mesh.insert(pos);
        for dir in Direction::ALL {
            let nb = ChunkPos::from(pos + dir.offset());
            if chunk_map.is_loaded(&nb) {
                to_mesh.insert(nb);
            }
        }
    }

    for pos in to_mesh {
        let Some(chunk) = chunk_map.get(&pos) else {
            continue;
        };

        if chunk.storage.is_empty() {
            continue;
        };

        let input = MeshInput::build(pos, chunk.storage.clone(), &chunk_map, model_cache.clone());

        let task = pool.spawn(async move { mesh_chunk(input) });

        match entity_map.0.get(&pos) {
            Some(&root) => {
                commands.entity(root).insert(PendingMeshTask(task));
            }
            None => {
                let root = commands
                    .spawn((
                        ChunkMeshRoot(pos),
                        PendingMeshTask(task),
                        Transform::from_translation(pos.into_world_pos()),
                        Visibility::default(),
                    ))
                    .id();
                entity_map.0.insert(pos, root);
            }
        }
    }
}
