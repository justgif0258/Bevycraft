use crate::prelude::{CHUNK_SIZE, Chunk, ChunkLoaderConfig, ChunkMap, ChunkPos, GeneratorResource};
use bevy::math::{IVec3, Vec3};
use bevy::prelude::{Component, Message, MessageWriter, Query, Res, ResMut, Transform, With};
use bevy::tasks::AsyncComputeTaskPool;
use bevy::tasks::futures::check_ready;

#[derive(Message, Debug, Copy, Clone)]
pub struct ChunkReadyEvent(pub ChunkPos);

#[derive(Message, Debug, Copy, Clone)]
pub struct ChunkUnloadedEvent(pub ChunkPos);

pub fn update_load_queue(
    loaders: Query<&Transform, With<ChunkLoader>>,
    config: Res<ChunkLoaderConfig>,
    mut chunk_map: ResMut<ChunkMap>,
) {
    let view = config.view_distance;
    let view_sq = view * view;
    let unload_sq = (view + config.unload_margin) * (view_sq + config.unload_margin);

    let origins: Vec<IVec3> = loaders
        .iter()
        .map(|t| world_to_chunk(t.translation))
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
                        chunk_map.enqueue(ChunkPos::from(origin + IVec3::new(dx, dy, dz)), dist_sq);
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
        chunk_map.pending.insert(pos, task);
    }
}

pub fn poll_chunk_tasks(
    mut chunk_map: ResMut<ChunkMap>,
    mut ready_msg: MessageWriter<ChunkReadyEvent>,
) {
    let mut completed: Vec<(ChunkPos, Chunk)> = Vec::new();

    chunk_map
        .pending
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
        ready_msg.write(ChunkReadyEvent(pos));
    }
}

pub fn process_unload_queue(
    mut chunk_map: ResMut<ChunkMap>,
    mut unload_msg: MessageWriter<ChunkUnloadedEvent>,
) {
    const MAX_PER_FRAME: usize = 8;

    for _ in 0..MAX_PER_FRAME {
        let Some(pos) = chunk_map.unload_queue.pop_front() else {
            break;
        };

        if chunk_map.remove(&pos).is_some() {
            unload_msg.write(ChunkUnloadedEvent(pos));
        }
    }
}

#[derive(Component, Default)]
pub struct ChunkLoader;

#[inline(always)]
fn world_to_chunk(world: Vec3) -> IVec3 {
    (world / CHUNK_SIZE as f32).floor().as_ivec3()
}
