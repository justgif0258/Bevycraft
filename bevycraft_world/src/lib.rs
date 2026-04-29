mod block;
mod chunk;
mod morton;
mod generator;

pub mod prelude {
    #[allow(deprecated)]
    pub use crate::{
        block::{
            block::{Block, AttachmentAttribute, Attachments},
            block_record::*,
            block_commit::BlockCommit,
            block_flags::BlockFlags,
            block_behaviour::BlockBehaviour,
        },
        chunk::{
            chunk::*,
        },
        morton::morton_3d::{
            Morton3D,
            MortonEncodable,
            MortonDecodable
        },
        generator::{
            chunk_generator::*,
            simple_generator::SimpleGenerator,
        },
    };
}
