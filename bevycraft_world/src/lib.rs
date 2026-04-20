mod block;
mod chunk;
mod morton;
mod spatial;
mod generator;

pub mod prelude {
    #[allow(deprecated)]
    pub use crate::{
        block::{
            block::{Block, AttachmentAttribute, Attachments},
            block_record::BlockRecord,
            block_commit::BlockCommit,
            block_flags::BlockFlags,
            block_behaviour::BlockBehaviour,
        },
        chunk::{
            section::*,
            chunk::*,
            pool::*,
            manager::*,
        },
        morton::morton_3d::{
            Morton3D,
            MortonEncodable,
            MortonDecodable
        },
        spatial::{
            node_64::Node64, 
            tree_64::Tree64
        },
        generator::{
            world_generator::*,
            basic_generator::BasicGenerator,
        },
    };
}
