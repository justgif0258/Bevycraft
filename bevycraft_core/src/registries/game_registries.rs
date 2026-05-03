use std::sync::OnceLock;

use crate::prelude::{Block, DefaultedRegistry};

static BLOCKS: OnceLock<DefaultedRegistry<Block>> = OnceLock::new();

pub struct GameRegistries;

impl GameRegistries {
    #[inline(always)]
    pub fn blocks() -> &'static DefaultedRegistry<Block> {
        BLOCKS.get().expect("Cannot access uninitialized registry")
    }

    #[inline(always)]
    pub fn init_blocks(registry: DefaultedRegistry<Block>) {
        BLOCKS.set(registry).ok().unwrap();
    }
}
