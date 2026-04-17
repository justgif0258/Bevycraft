use std::sync::LazyLock;
use bevy::platform::collections::HashMap;
use ron::extensions::Extensions;
use serde::{Deserialize, Serialize};
use bevycraft_core::prelude::AssetLocation;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RModel {
    pub parent  : Option<AssetLocation>,
    pub textures: Option<HashMap<String, AssetLocation>>,
    pub elements: Option<Vec<Element>>,
}

impl RModel {
    #[inline]
    pub fn from_str(model: &str) -> Result<RModel, String> {
        static OPTIONS: LazyLock<ron::Options> = LazyLock::new(|| {
            ron::Options::default()
                .with_default_extension(Extensions::IMPLICIT_SOME)
        });

        OPTIONS.from_str(model)
            .map_err(|e| e.to_string())
    }
    
    #[inline]
    pub fn textures(&self) -> Vec<&AssetLocation> {
        if let Some(textures) = &self.textures { 
            return textures.values()
                .collect();
        }
        
        Vec::new()
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Element {
    pub from    : [f32; 3],
    pub to      : [f32; 3],
    pub rotation: Option<Rotation>,
    pub faces   : HashMap<String, Face>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Face {
    pub uv      : [f32; 4],
    pub texture : String,
    pub cullface: Option<String>,

    #[serde(default)]
    pub tintable: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Rotation {
    pub origin  : [f32; 3],
    pub x       : f32,
    pub y       : f32,
    pub z       : f32,
}