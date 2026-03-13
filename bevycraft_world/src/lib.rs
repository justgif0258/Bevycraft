mod block;
mod morton;
mod chunk;
mod spatial;

pub mod prelude {
    pub use crate::block::{
        block::{
            Block,
            BlockBehaviour,
            BehaviourTrait
        },
    };
    pub use crate::morton::morton_3d::*;
    #[allow(deprecated)]
    pub use crate::spatial::{
        node_64::Node64,
        tree_64::Tree64,
    };
}

pub mod presets {
    pub use crate::block::{
        *
    };
}