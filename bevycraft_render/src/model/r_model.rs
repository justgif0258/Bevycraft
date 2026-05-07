use std::sync::LazyLock;

use bevy::{
    app::{App, Plugin},
    asset::{Asset, AssetApp, AssetLoader, LoadContext, io::Reader},
    platform::collections::HashMap,
    reflect::TypePath,
};
use bevycraft_core::prelude::AssetLocation;
use ron::{Options, extensions::Extensions};
use serde::Deserialize;

use crate::prelude::{Direction, RenderMode};

pub struct RModelPlugin;

impl Plugin for RModelPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<RModel>()
            .init_asset_loader::<RModelLoader>();
    }
}

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

        static RON_READER: LazyLock<Options> =
            LazyLock::new(|| Options::default().with_default_extension(Extensions::IMPLICIT_SOME));

        let unsolved_model: UnresolvedRModel = RON_READER
            .from_bytes(&bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let mut elements = unsolved_model.elements.unwrap_or_default();
        let mut textures = unsolved_model.textures.unwrap_or_default();

        if let Some(parent) = unsolved_model.parent {
            let path = format!("{}/models/{}.ron", parent.namespace(), parent.path());

            let loaded = load_context
                .loader()
                .immediate()
                .load::<RModel>(path)
                .await
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

            let parent = loaded.get();

            if elements.is_empty() {
                elements = parent.elements.clone();
            }

            if textures.is_empty() {
                textures = parent.textures.clone();
            }
        }

        Ok(RModel { textures, elements })
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

#[derive(Deserialize)]
struct UnresolvedRModel {
    parent: Option<AssetLocation>,
    textures: Option<HashMap<String, AssetLocation>>,
    elements: Option<Vec<Element>>,
}

#[derive(Asset, TypePath, Debug, Clone)]
pub struct RModel {
    pub textures: HashMap<String, AssetLocation>,
    pub elements: Vec<Element>,
}

impl RModel {
    #[inline]
    pub fn textures(&self) -> impl Iterator<Item = &AssetLocation> {
        self.textures.values()
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Element {
    pub from: [f32; 3],
    pub to: [f32; 3],
    pub rotation: Option<Rotation>,
    pub faces: HashMap<Direction, Face>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Face {
    pub uv: [f32; 4],
    pub texture: String,
    pub cullface: Option<Direction>,

    #[serde(default)]
    pub render_mode: RenderMode,

    #[serde(default)]
    pub tintable: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Rotation {
    pub origin: [f32; 3],
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
