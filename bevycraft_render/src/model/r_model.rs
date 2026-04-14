use std::sync::LazyLock;
use bevy::platform::collections::HashMap;
use ron::extensions::Extensions;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RModel {
    pub parent  : Option<String>,
    pub textures: Option<HashMap<String, String>>,
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
    pub fn textures(&self) -> Vec<&str> {
        if let Some(textures) = &self.textures { 
            return textures.values()
                .map(|t| t.as_str())
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
    pub cullface: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Rotation {
    pub origin  : [f32; 3],
    pub x       : f32,
    pub y       : f32,
    pub z       : f32,
}