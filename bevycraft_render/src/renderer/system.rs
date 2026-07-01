use {
    crate::{
        prelude::{
            mesh_chunk, ArrayTexture, BlockModel, ChunkEntityMap, ChunkMeshLayer, ChunkMeshRoot,
            Direction, MeshInput, ModelCache, RenderMode,
        },
        renderer::component::{BatchOutput, InflightBatch, MeshingQueue},
    },
    bevy::{
        camera::primitives::{Aabb, Frustum},
        platform::collections::HashSet,
        prelude::{
            Assets, Children, Commands, Mesh, Mesh3d, MeshMaterial3d, MessageReader, Query, Res,
            ResMut, Transform, Vec3, Visibility,
        },
        tasks::{futures::check_ready, AsyncComputeTaskPool},
    },
    bevycraft_core::prelude::Block,
    bevycraft_world::prelude::{ChunkMap, ChunkPos, ChunkReady, ChunkUnloaded, CHUNK_SIZE},
};

pub fn trigger_chunk_meshing(
    mut queue: ResMut<MeshingQueue>,
    mut events: MessageReader<ChunkReady>,
    chunk_map: Res<ChunkMap>,
) {
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

    for pos in positions {
        queue.pending.insert(pos);
    }
}

pub fn dispatch_mesh_tasks(
    mut queue: ResMut<MeshingQueue>,
    chunk_map: Res<ChunkMap>,
    model_cache: Res<ModelCache<Block, BlockModel>>,
    camera: Query<&Frustum>,
) {
    if queue.pending.is_empty() {
        return;
    }

    let pool = AsyncComputeTaskPool::get();
    let parallelism = pool.thread_num().max(1);
    let budget = queue.budget;

    let frustum = camera.single().ok();

    let snapshot: Vec<ChunkPos> = queue.pending.iter().copied().collect();
    let mut entries: Vec<(ChunkPos, MeshInput)> = Vec::with_capacity(budget);

    for &pos in &snapshot {
        if entries.len() >= budget {
            break;
        }

        if queue.inflight_chunks.contains(&pos) {
            continue;
        }

        if let Some(frustum) = frustum {
            let world_pos = pos.into_world_pos();
            let aabb = Aabb::from_min_max(world_pos, world_pos + Vec3::splat(CHUNK_SIZE as f32));
            if !frustum.intersects_obb_identity(&aabb) {
                continue;
            }
        }

        let Some(chunk) = chunk_map.get(&pos) else {
            queue.pending.remove(&pos);
            continue;
        };
        if chunk.storage.is_empty() {
            queue.pending.remove(&pos);
            continue;
        };

        let input = MeshInput::build(pos, chunk.storage.clone(), &chunk_map, model_cache.clone());
        entries.push((pos, input));
        queue.pending.remove(&pos);
    }

    if entries.is_empty() {
        return;
    }

    for &(pos, _) in &entries {
        queue.inflight_chunks.insert(pos);
    }

    let batch_size = (entries.len() + parallelism - 1) / parallelism;

    for batch in entries.chunks(batch_size) {
        let batch_owned = batch.to_vec();

        let task = pool.spawn(async move {
            let results = batch_owned
                .into_iter()
                .map(|(pos, input)| (pos, mesh_chunk(input)))
                .collect();

            BatchOutput { results }
        });

        queue.inflight_batches.push(InflightBatch { task });
    }
}

pub fn poll_mesh_tasks(
    mut commands: Commands,
    mut queue: ResMut<MeshingQueue>,
    mut entity_map: ResMut<ChunkEntityMap>,
    mut meshes: ResMut<Assets<Mesh>>,
    textures: Res<ArrayTexture>,
) {
    let mut completed_outputs: Vec<BatchOutput> = Vec::new();

    queue.inflight_batches.retain_mut(|batch| {
        match check_ready(&mut batch.task) {
            Some(output) => {
                completed_outputs.push(output);
                false // remove from inflight
            }
            None => true, // keep
        }
    });

    for output in completed_outputs {
        for (pos, chunk_output) in output.results {
            if !queue.inflight_chunks.remove(&pos) {
                continue;
            }

            // Create the root entity on first mesh, or reuse it on remesh.
            let root_entity = match entity_map.0.get(&pos) {
                Some(&entity) => entity,
                None => {
                    let entity = commands
                        .spawn((
                            ChunkMeshRoot(pos),
                            Transform::from_translation(pos.into_world_pos()),
                            Visibility::default(),
                        ))
                        .id();
                    entity_map.0.insert(pos, entity);
                    entity
                }
            };

            commands.entity(root_entity).despawn_related::<Children>();

            for (mesh_opt, mode) in [
                (chunk_output.opaque, RenderMode::Opaque),
                (chunk_output.cutout, RenderMode::Cutout),
                (chunk_output.translucent, RenderMode::Translucent),
            ] {
                let Some(mesh) = mesh_opt else {
                    continue;
                };

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
}

pub fn cleanup_chunk_entities(
    mut commands: Commands,
    mut events: MessageReader<ChunkUnloaded>,
    mut entity_map: ResMut<ChunkEntityMap>,
    mut queue: ResMut<MeshingQueue>,
) {
    for &ChunkUnloaded(pos) in events.read() {
        if let Some(root) = entity_map.0.remove(&pos) {
            commands.entity(root).despawn();
        }
        queue.pending.remove(&pos);
        queue.inflight_chunks.remove(&pos);
    }
}

pub fn remesh_dirty_chunks(mut queue: ResMut<MeshingQueue>, mut chunk_map: ResMut<ChunkMap>) {
    let mut dirty: Vec<ChunkPos> = Vec::new();

    for (&pos, chunk) in chunk_map.chunks.iter_mut() {
        if chunk.dirty {
            chunk.dirty = false;
            dirty.push(pos);
        }
    }

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
        queue.pending.insert(pos);
    }
}
