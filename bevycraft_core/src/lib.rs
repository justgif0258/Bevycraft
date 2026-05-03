extern crate core;

mod block;
mod memory;
mod registries;
mod util;

pub mod prelude {
    pub use crate::block::{BlockType, block::*, block_behaviour::*, block_flags::*};
    pub use crate::memory::packed_array_u32::PackedArrayU32;
    pub use crate::registries::{
        asset_location::*, defaulted_registry::*, erased_registry::*, game_registries::*,
        ordered_registry::*, registry::*,
    };
}
