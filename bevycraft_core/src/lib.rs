extern crate core;

mod block;
pub mod consts;
mod memory;
mod registries;
mod util;

pub mod prelude {
    pub use crate::{
        block::{behaviour::*, block::*, flags::*, shape::BlockShape},
        memory::pattern_container::{PatternContainer, PatternIter},
        registries::{
            asset_location::*, defaulted_registry::*, holder::Holder, ordered_registry::*,
            registrar::*, registry::*,
        },
    };
}

pub mod blocks {
    pub use crate::block::blocks::*;
}
