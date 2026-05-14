use {
    crate::prelude::{Chunk, ChunkLoaderConfig, ChunkMap, ChunkPos, GeneratorResource},
    bevy::{
        prelude::{Component, Message, MessageWriter, Query, Res, ResMut, Transform, With},
        tasks::{futures::check_ready, AsyncComputeTaskPool},
    },
};

#[derive(Message, Debug, Copy, Clone)]
pub struct ChunkReady(pub ChunkPos);

#[derive(Message, Debug, Copy, Clone)]
pub struct ChunkUnloaded(pub ChunkPos);

#[derive(Component, Default)]
pub struct ChunkLoader;

pub fn update_queue(
    loaders: Query<&Transform, With<ChunkLoader>>,
    config: Res<ChunkLoaderConfig>,
    mut chunk_map: ResMut<ChunkMap>,
) {
    let view = config.view_distance;
    let view_sq = view * view;
    let unload_sq = (view + config.unload_margin) * (view + config.unload_margin);

    let origins: Vec<ChunkPos> = loaders
        .iter()
        .map(|t| ChunkPos::from_world_pos(t.translation))
        .collect();

    if origins.is_empty() {
        return;
    }

    for &origin in &origins {
        for dx in -view..view {
            for dy in -view..view {
                for dz in -view..view {
                    let dist_sq = dx * dx + dy * dy + dz * dz;

                    if dist_sq <= view_sq {
                        chunk_map
                            .enqueue(ChunkPos::from(origin + ChunkPos::new(dx, dy, dz)), dist_sq);
                    }
                }
            }
        }
    }

    let loaded: Vec<ChunkPos> = chunk_map.chunks.keys().copied().collect();

    for pos in loaded {
        let min_dist_sq = origins
            .iter()
            .map(|&o| {
                let d = pos - o;
                d.x * d.x + d.y * d.y + d.z * d.z
            })
            .min()
            .unwrap_or(i32::MAX);

        if min_dist_sq > unload_sq {
            chunk_map.enqueue_unload(pos);
        }
    }
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
    const MAX_PER_FRAME: usize = 8;

    for _ in 0..MAX_PER_FRAME {
        let Some(pos) = chunk_map.unload_queue.pop_front() else {
            break;
        };

        if chunk_map.remove(&pos).is_some() {
            unload_msg.write(ChunkUnloaded(pos));
        }
    }
}
