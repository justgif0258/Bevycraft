use std::path::Path;
use bevy::log::{info, warn};
use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::{error, Resource};
use bevycraft_core::prelude::AssetLocation;
use crate::model::r_model::RModel;
use crate::prelude::Element;

#[derive(Resource, Debug, Default)]
pub struct RModelManager {
    models: HashMap<AssetLocation, RModel>,
}

impl RModelManager {
    #[inline]
    pub fn load(&mut self, location: AssetLocation) {
        if self.is_loaded(&location) {
            warn!("Tried loading model from {}, which was already loaded. Skipping...", location);

            return;
        }

        let ron = get_ron(format!("bevycraft_app/assets/{}/models/{}.ron", location.namespace(), location.path()));

        if ron.is_none() {
            error!("Couldn't find model {}. Skipping...", location);

            return;
        }

        let result = RModel::from_str(&ron.unwrap());

        if result.is_err() {
            error!("Failed to load model {} ({})", location, result.err().unwrap());

            return;
        }

        info!("Successfully loaded model from {}", location);

        self.models.insert(location, result.unwrap());
    }

    #[inline]
    pub fn try_load_parent(&self, model: &RModel) -> Option<&Vec<Element>> {
        if let Some(parent) = &model.parent {
            return self.models.get(parent)
                .map(|model| model.elements.as_ref())?;
        }

        None
    }
    
    #[inline]
    pub fn get_textures_locations(&self) -> HashSet<AssetLocation> {
        let mut result: HashSet<AssetLocation> = HashSet::new();
        
        self.models.iter()
            .for_each(|(_, m)| {
                m.textures()
                    .into_iter()
                    .for_each(|t| { 
                        result.insert(t.clone());
                    })
            });
        
        result
    }

    #[inline(always)]
    fn is_loaded(&self, location: &AssetLocation) -> bool {
        self.models.contains_key(location)
    }
}

#[inline(always)]
fn get_ron(path: impl AsRef<Path>) -> Option<String> {
    std::fs::read_to_string(path).ok()
}