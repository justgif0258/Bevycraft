use {
    crate::prelude::{Chunk, ChunkLoaderConfig, ChunkMap, ChunkPos, GeneratorResource},
    bevy::{
        platform::{collections::HashSet, hash::NoOpHash},
        prelude::{
            Component, Message, MessageWriter, Query, Res, ResMut, Resource, Transform, With,
        },
        tasks::{AsyncComputeTaskPool, futures::check_ready},
    },
};

#[derive(Message, Debug, Copy, Clone)]
pub struct ChunkReady(pub ChunkPos);

#[derive(Message, Debug, Copy, Clone)]
pub struct ChunkUnloaded(pub ChunkPos);

#[derive(Component, Default)]
pub struct ChunkLoader;

#[derive(Resource)]
pub struct ViewVolume {
    target: HashSet<ChunkPos, NoOpHash>,
    last_origins: Vec<ChunkPos>,
}

impl ViewVolume {
    pub const fn new() -> Self {
        Self {
            target: HashSet::with_hasher(NoOpHash),
            last_origins: Vec::new(),
        }
    }
}

pub fn update_queue(
    loaders: Query<&Transform, With<ChunkLoader>>,
    config: Res<ChunkLoaderConfig>,
    mut chunk_map: ResMut<ChunkMap>,
    mut view_volume: ResMut<ViewVolume>,
) {
    let view = config.view_distance;
    let view_sq = view * view;

    let origins: Vec<ChunkPos> = loaders
        .iter()
        .map(|t| ChunkPos::from_world_pos(t.translation))
        .collect();

    if origins == view_volume.last_origins {
        return;
    }

    let old_target = std::mem::take(&mut view_volume.target);

    for &origin in &origins {
        for dx in -view..=view {
            for dy in -view..=view {
                for dz in -view..=view {
                    let dist_sq = dx * dx + dy * dy + dz * dz;

                    if dist_sq <= view_sq {
                        view_volume.target.insert(ChunkPos::new(
                            origin.x + dx,
                            origin.y + dy,
                            origin.z + dz,
                        ));
                    }
                }
            }
        }
    }

    for &pos in &view_volume.target {
        if !old_target.contains(&pos) {
            let min_dist_sq = origins
                .iter()
                .map(|&o| {
                    let d = pos - o;

                    d.x * d.x + d.y * d.y + d.z * d.z
                })
                .min()
                .unwrap_or(i32::MAX);

            chunk_map.enqueue(pos, min_dist_sq);
        }
    }

    for &pos in &old_target {
        if !view_volume.target.contains(&pos) {
            chunk_map.enqueue_unload(pos);
        }
    }

    view_volume.last_origins = origins;
}

pub fn spawn_chunk_tasks(mut chunk_map: ResMut<ChunkMap>, generator: Res<GeneratorResource>) {
    let budget = chunk_map
        .max_concurrent
        .saturating_sub(chunk_map.pending_count());

    if budget == 0 {
        return;
    }

    let pool = AsyncComputeTaskPool::get();

    for _ in 0..budget {
        let Some(req) = chunk_map.load_queue.pop() else {
            break;
        };

        if chunk_map.chunks.contains_key(&req.pos) {
            chunk_map.inflight.remove(&req.pos);

            continue;
        }

        let generator = generator.0.clone();
        let pos = req.pos;

        let task = pool.spawn(async move { generator.generate(pos) });

        chunk_map.pending_load.insert(pos, task);
    }
}

pub fn poll_chunk_tasks(mut chunk_map: ResMut<ChunkMap>, mut ready_msg: MessageWriter<ChunkReady>) {
    let mut completed: Vec<(ChunkPos, Chunk)> = Vec::new();

    chunk_map
        .pending_load
        .retain(|&pos, task| match check_ready(task) {
            None => true,
            Some(chunk) => {
                completed.push((pos, chunk));

                false
            }
        });

    for (pos, chunk) in completed {
        chunk_map.inflight.remove(&pos);
        chunk_map.chunks.insert(pos, chunk);
        ready_msg.write(ChunkReady(pos));
    }
}

pub fn process_unload_queue(
    mut chunk_map: ResMut<ChunkMap>,
    mut unload_msg: MessageWriter<ChunkUnloaded>,
) {
    loop {
        let Some(pos) = chunk_map.unload_queue.pop_front() else {
            break;
        };

        if chunk_map.remove(&pos).is_some() {
            unload_msg.write(ChunkUnloaded(pos));
        }
    }
}
