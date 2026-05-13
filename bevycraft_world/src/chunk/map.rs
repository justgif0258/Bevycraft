use bevy::platform::collections::HashMap;
use crate::prelude::{Chunk, ChunkPos};

pub struct ChunkMap {
    chunks: HashMap<ChunkPos, Chunk>,
}