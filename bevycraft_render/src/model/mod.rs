use bevy::{asset::Asset, reflect::TypePath};

use crate::prelude::*;

pub mod block_model;
pub mod model_cache;
pub mod r_model;

pub trait Model: Asset + TypePath {
    type Raw: for<'de> serde::Deserialize<'de>;

    fn resolve(raw: Self::Raw, textures: &TextureRegistry) -> Self;
}
