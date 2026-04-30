use std::sync::OnceLock;

use crate::prelude::{Block, DefaultedRegistry, ErasedRegistry, OrderedRegistry};

pub static ROOT: OnceLock<OrderedRegistry<&'static dyn ErasedRegistry>> = OnceLock::new();

pub static BLOCKS: OnceLock<DefaultedRegistry<Block>> = OnceLock::new();

pub struct CoreRegistries;

impl CoreRegistries {
    #[inline(always)]
    pub fn blocks() -> &'static DefaultedRegistry<Block> {
        BLOCKS.get().expect("Cannot access uninitialized registry")
    }

    #[inline(always)]
    pub fn init_blocks(registry: DefaultedRegistry<Block>) {
        BLOCKS.set(registry).ok().unwrap();
    }
}
