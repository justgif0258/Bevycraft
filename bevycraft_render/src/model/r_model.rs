use bevy::{
    asset::{Asset, AssetLoader, Handle, LoadContext, io::Reader},
    log::info,
    platform::collections::{HashMap, hash_map::Values},
    reflect::TypePath,
};
use bevycraft_core::prelude::AssetLocation;
use serde::{Deserialize, Serialize};

#[derive(Default, TypePath)]
pub struct RModelLoader;

impl AssetLoader for RModelLoader {
    type Asset = RModel;
    type Settings = ();
    type Error = std::io::Error;

    #[inline]
    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();

        reader.read_to_end(&mut bytes).await?;

        let unsolved_model: UnsolvedRModel = ron::de::from_bytes(&bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let parent = unsolved_model.parent.map(|p| {
            let path = format!("{}/models/{}.ron", p.namespace(), p.path());

            info!("Loading parent model: {}", path);

            load_context.load::<RModel>(path)
        });

        Ok(RModel {
            parent,
            textures: unsolved_model.textures,
            elements: unsolved_model.elements,
        })
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

#[derive(Deserialize)]
struct UnsolvedRModel {
    parent: Option<AssetLocation>,
    textures: Option<HashMap<String, AssetLocation>>,
    elements: Option<Vec<Element>>,
}

#[derive(Asset, TypePath, Debug, Clone)]
pub struct RModel {
    pub parent: Option<Handle<RModel>>,
    pub textures: Option<HashMap<String, AssetLocation>>,
    pub elements: Option<Vec<Element>>,
}

impl RModel {
    #[inline]
    pub fn textures(&self) -> impl Iterator<Item = &AssetLocation> {
        if let Some(textures) = &self.textures {
            return textures.values();
        }

        Values::default()
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Element {
    pub from: [f32; 3],
    pub to: [f32; 3],
    pub rotation: Option<Rotation>,
    pub faces: HashMap<String, Face>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Face {
    pub uv: [f32; 4],
    pub texture: String,
    pub cullface: Option<String>,

    #[serde(default = "default_rendering_mode")]
    pub render_mode: String,

    #[serde(default)]
    pub tintable: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Rotation {
    pub origin: [f32; 3],
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

fn default_rendering_mode() -> String {
    String::from("opaque")
}
