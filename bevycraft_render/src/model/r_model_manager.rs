use std::path::Path;
use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::*;
use bevycraft_core::prelude::AssetLocation;
use crate::prelude::*;

#[derive(Resource, Debug, Default)]
pub struct RModelManager {
    models: HashMap<AssetLocation, RModel>,
}

impl RModelManager {
    #[inline]
    pub fn load(&mut self, location: AssetLocation) -> Result<(), String> {
        if self.is_loaded(&location) {
            return Err(format!("Model {} already loaded", location));
        }

        let path = format!("bevycraft_app/assets/{}/models/{}.ron", location.namespace(), location.path());

        let ron = get_ron(path);

        if ron.is_none() {
            return Err(format!("Couldn't find model {}. Skipping...", location));
        }

        let result = RModel::from_str(&ron.unwrap());

        match result {
            Err(e) => Err(format!("Failed to load model {}: {}", location, e)),
            Ok(model) => {
                if let Some(dependency) = &model.parent && !self.is_loaded(dependency) {
                    let path = format!("bevycraft_app/assets/{}/models/{}.ron", dependency.namespace(), dependency.path());

                    if let Some(parent) = get_ron(path) {
                        match RModel::from_str(&parent) {
                            Err(e) => return Err(format!("Failed to load dependency {}: {}", dependency, e)),
                            Ok(parent) => {
                                self.models.insert(dependency.clone(), parent);
                            }
                        }
                    }
                }

                self.models.insert(location, model);

                Ok(())
            }
        }
    }

    #[inline]
    pub fn take(&mut self, location: &AssetLocation) -> Option<RModel> {
        self.models.remove(location)
    }

    #[inline]
    pub fn get(&self, location: &AssetLocation) -> Option<&RModel> {
        self.models.get(location)
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
            .for_each(|(_, model)| {
                model.textures()
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