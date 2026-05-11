use std::marker::PhantomData;

use bevy::{
    app::{App, Plugin},
    asset::{AssetApp, AssetLoader, LoadContext, io::Reader},
    platform::collections::HashMap,
    reflect::TypePath,
};
use bevycraft_core::prelude::AssetLocation;
use ron::Options;
use serde::Deserialize;

use crate::{
    model::{Model, ModelLoadError},
    prelude::{Direction, RenderMode},
    textures::texture_manager::TextureManager,
};

pub struct RModelPlugin<M: Model>(PhantomData<M>);

impl<M: Model> Default for RModelPlugin<M> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<M: Model> Plugin for RModelPlugin<M> {
    fn build(&self, app: &mut App) {
        let manager = TextureManager::<M>::default();

        app.insert_resource(manager.clone())
            .init_asset::<M>()
            .register_asset_loader(RModelLoader::<M> { manager });
    }
}

#[derive(TypePath)]
pub struct RModelLoader<M: Model> {
    manager: TextureManager<M>,
}

impl<M: Model> AssetLoader for RModelLoader<M> {
    type Asset = M;
    type Settings = Options;
    type Error = ModelLoadError<M::Error>;

    #[inline]
    async fn load(
        &self,
        reader: &mut dyn Reader,
        settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();

        reader.read_to_end(&mut bytes).await?;

        let mut ron = settings.from_bytes::<UnresolvedRModel>(&bytes)?;

        if let Some(parent) = &ron.parent {
            let loaded = load_context
                .read_asset_bytes(parent.clone())
                .await
                .map_err(ModelLoadError::ReadBytesError)?;

            let parent = settings.from_bytes::<UnresolvedRModel>(&loaded)?;

            if ron.elements.is_empty() {
                ron.elements = parent.elements.clone();
            }

            if ron.textures.is_empty() {
                ron.textures = parent.textures.clone();
            }
        }

        let mut elements: Vec<Element> = Vec::with_capacity(ron.elements.len());

        for element in ron.elements {
            let mut faces: HashMap<Direction, Face> = HashMap::with_capacity(element.faces.len());

            for (dir, face) in element.faces {
                let texture = match face.texture.strip_prefix('#') {
                    Some(k) => ron.textures.get(k).cloned(),
                    None => Some(
                        AssetLocation::try_parsing(&face.texture)
                            .map_err(|e| ModelLoadError::InvalidLocation(e))?,
                    ),
                };

                if texture.is_none() {
                    return Err(ModelLoadError::UndefinedTexture(face.texture));
                }

                faces.insert(
                    dir,
                    Face {
                        uv: face.uv,
                        texture: texture.unwrap(),
                        cullface: face.cullface,
                        render_mode: face.render_mode,
                        tintable: face.tintable,
                    },
                );
            }

            let resolved = Element {
                from: element.from,
                to: element.to,
                rotation: element.rotation,
                faces,
            };

            elements.push(resolved);
        }

        M::resolve(RModel { elements }, &self.manager)
            .await
            .map_err(ModelLoadError::Resolve)
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

#[derive(Debug, Clone)]
pub struct RModel {
    pub elements: Vec<Element>,
}

#[derive(Debug, Clone)]
pub struct Element {
    pub from: [f32; 3],
    pub to: [f32; 3],
    pub rotation: Option<Rotation>,
    pub faces: HashMap<Direction, Face>,
}

#[derive(Debug, Clone)]
pub struct Face {
    pub uv: [f32; 4],
    pub texture: AssetLocation,
    pub cullface: Option<Direction>,
    pub render_mode: RenderMode,
    pub tintable: bool,
}

#[derive(Deserialize, Debug, Clone)]
struct UnresolvedRModel {
    pub parent: Option<AssetLocation>,

    #[serde(default)]
    pub textures: HashMap<Box<str>, AssetLocation>,

    #[serde(default)]
    pub elements: Vec<UnresolvedElement>,
}

#[derive(Deserialize, Debug, Clone)]
struct UnresolvedElement {
    pub from: [f32; 3],
    pub to: [f32; 3],
    pub rotation: Option<Rotation>,
    pub faces: HashMap<Direction, UnresolvedFace>,
}

#[derive(Deserialize, Debug, Clone)]
struct UnresolvedFace {
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
