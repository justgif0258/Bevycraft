use crate::prelude::{BlockFlags, Definition};

pub mod definition;
pub mod block_definition;
pub mod block_flags;
pub mod block_physics;

pub static HARDNESS: Definition<f32> = Definition::new("hardness", 0.0)
    .with_normalizer(|v| v.max(0.0));

pub static TOUGHNESS: Definition<f32> = Definition::new("toughness", 0.0)
    .with_normalizer(|v| v.max(0.0));

pub static FRICTION: Definition<f32> = Definition::new("friction", 0.6)
    .with_normalizer(|v| v.max(0.0));

pub static VISCOSITY: Definition<f32> = Definition::new("viscosity", 0.0)
    .with_normalizer(|v| v.clamp(0.0, 1.0));

pub static EMISSION: Definition<f32> = Definition::new("emission", 0.0)
    .with_normalizer(|v| v.max(0.0));

pub static FLAGS: Definition<BlockFlags> = Definition::new("flags", BlockFlags::AIR);