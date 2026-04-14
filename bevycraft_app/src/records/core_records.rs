use std::sync::OnceLock;
use bevycraft_world::prelude::BlockRecord;

pub static BLOCKS: OnceLock<BlockRecord> = OnceLock::new();

pub fn blocks() -> &'static BlockRecord {
    BLOCKS.get()
        .expect("Tried accessing blocks record, but it wasn't initialized")
}