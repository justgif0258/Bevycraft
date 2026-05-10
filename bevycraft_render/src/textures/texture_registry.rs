use std::{collections::BTreeMap, sync::Arc};

use bevy::ecs::resource::Resource;
use bevycraft_core::prelude::AssetLocation;
use parking_lot::Mutex;

use crate::prelude::TextureId;

#[derive(Resource, Debug, Default, Clone)]
pub struct TextureRegistry(Arc<Mutex<BTreeMap<AssetLocation, TextureId>>>);

impl TextureRegistry {
    pub fn get_or_insert(&self, location: &AssetLocation) -> TextureId {
        let mut map = self.0.lock();

        if let Some(&texture_id) = map.get(location) {
            return texture_id;
        }

        let texture_id = TextureId(map.len() as u32);
        map.insert(location.clone(), texture_id);
        texture_id
    }
}
