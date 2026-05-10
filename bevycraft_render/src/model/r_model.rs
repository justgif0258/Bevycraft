use std::{
    collections::BTreeMap,
    sync::{Arc, LazyLock, Mutex},
};

use bevy::{
    app::{App, Plugin},
    asset::{Asset, AssetApp, AssetLoader, LoadContext, io::Reader},
    platform::collections::HashMap,
    reflect::TypePath,
};
use bevycraft_core::prelude::AssetLocation;
use ron::{Options, extensions::Extensions};
use serde::Deserialize;

use crate::{
    model::{Model, block_model::BlockModel},
    prelude::{Direction, RenderMode},
    textures::texture_registry::TextureRegistry,
};

pub struct RModelPlugin;

impl Plugin for RModelPlugin {
    fn build(&self, app: &mut App) {
        let textures = TextureRegistry::default();

        app.insert_resource(textures.clone())
            .init_asset::<RModel>()
            .register_asset_loader(RModelLoader { textures });
    }
}

#[derive(Default, TypePath)]
pub struct RModelLoader {
    textures: TextureRegistry,
}

impl AssetLoader for RModelLoader {
    type Asset = BlockModel;
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

        let mut r_model: RModel = RON_READER
            .from_bytes(&bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        if let Some(parent) = &r_model.parent {
            let path = format!("{}/models/{}.ron", parent.namespace(), parent.path());

            let loaded = load_context
                .loader()
                .immediate()
                .load::<RModel>(path)
                .await
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

            let parent = loaded.get();

            if r_model.elements.is_empty() {
                r_model.elements = parent.elements.clone();
            }

            if r_model.textures.is_empty() {
                r_model.textures = parent.textures.clone();
            }
        }

        Ok(BlockModel::resolve(r_model, &self.textures))
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
pub struct RModel {
    pub parent: Option<AssetLocation>,

    #[serde(default)]
    pub textures: HashMap<Box<str>, AssetLocation>,

    #[serde(default)]
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
    pub texture: Box<str>,
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
