use std::sync::LazyLock;

use bevy::{
    asset::AssetLoader,
    platform::{collections::HashMap, hash::RandomState},
    reflect::TypePath,
};
use bevycraft_core::prelude::AssetLocation;
use dashmap::DashMap;
use ron::{Options, extensions::Extensions};
use serde::Deserialize;

use crate::prelude::*;

static RON_READER: LazyLock<Options> =
    LazyLock::new(|| Options::default().with_default_extension(Extensions::IMPLICIT_SOME));

#[derive(Default, TypePath)]
pub struct RModelLoader {
    textures: DashMap<AssetLocation, TextureId, RandomState>,
}

impl AssetLoader for RModelLoader {
    type Asset = RModel;
    type Settings = ();
    type Error = std::io::Error;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        settings: &Self::Settings,
        load_context: &mut bevy::asset::LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();

        reader.read_to_end(&mut bytes);

        let unsolved = RON_READER
            .from_bytes::<UnresolvedRModel>(&bytes)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let textures = unsolved.textures.unwrap_or_default();
        let elements = unsolved.elements.unwrap_or_default();

        if let Some(parent) = unsolved.parent {
            let path = format!("{}/models/{}.ron", parent.namespace(), parent.path());

            let loaded = load_context
                .loader()
                .immediate()
                .load::<RModel>(path)
                .await
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

            let parent = loaded.get();
        }

        let mut quads = Vec::new();

        elements.iter().for_each(|element| {
            let [x0, y0, z0] = element.from;
            let [x1, y1, z1] = element.to;

            element.faces.iter().for_each(|(&dir, face)| {
                let texture = {
                    if let Some(location) = textures.get(&face.texture) {
                        self.textures
                            .get(location)
                            .map_or(TextureId(0), |t| *t.value())
                    } else {
                        TextureId(0)
                    }
                };
            });
        });

        Ok(())
    }
}

#[derive(Deserialize)]
struct UnresolvedRModel<'a> {
    #[serde(borrow)]
    parent: Option<&'a str>,

    #[serde(borrow)]
    textures: Option<HashMap<&'a str, &'a str>>,

    #[serde(borrow)]
    elements: Option<Vec<Element<'a>>>,
}

#[derive(Deserialize, Debug, Clone)]
struct Element<'a> {
    from: [f32; 3],
    to: [f32; 3],
    rotation: Option<Rotation>,

    #[serde(borrow)]
    faces: HashMap<Direction, Face<'a>>,
}

#[derive(Deserialize, Debug, Clone)]
struct Face<'a> {
    uv: [f32; 4],

    #[serde(borrow)]
    texture: &'a str,

    cullface: Option<Direction>,

    #[serde(default)]
    render_mode: RenderMode,

    #[serde(default)]
    tintable: bool,
}

#[derive(Deserialize, Debug, Clone)]
struct Rotation {
    origin: [f32; 3],
    x: f32,
    y: f32,
    z: f32,
}
