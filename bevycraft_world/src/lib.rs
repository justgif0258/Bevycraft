mod block;
mod chunk;
mod morton;
mod spatial;

pub mod prelude {
    #[allow(deprecated)]
    pub use crate::{
        block::{
            property::{
                erased_property::ErasedProperty,
                boolean_property::BooleanProperty,
                enum_property::EnumProperty,
                integer_property::IntegerProperty,
            },
            definition::{
                block_definition::*,
                block_flags::BlockFlags,
            },
            block::Block,
            block_record::BlockRecord,
        },
        chunk::{
            section::Section,
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
    };
}
