use crate::prelude::{ArrayTexture, Model, TextureId, VertexMaterial};
use bevy::ecs::resource::Resource;
use bevy::ecs::system::SystemParam;
use bevy::prelude::{Asset, Assets, Commands, Image, Res, ResMut, TypePath};
use bevycraft_core::prelude::AssetLocation;
use parking_lot::RwLock;
use std::marker::PhantomData;
use std::{collections::BTreeMap, sync::Arc};

#[derive(SystemParam)]
pub struct TextureBakery<'w, 's, M: Model> {
    commands: Commands<'w, 's>,
    images: ResMut<'w, Assets<Image>>,
    mats: ResMut<'w, Assets<VertexMaterial>>,
    textures: Res<'w, TextureManager<M>>,
}

impl<'w, 's, M: Model> TextureBakery<'w, 's, M> {
    pub fn bake(&mut self, width: u32, height: u32) {
        let mut array_texture = ArrayTexture::new_uninit(width, height);

        let read = self.textures.0.read();

        read.textures.iter().for_each(|(location, _)| {
            array_texture.load_from_asset_location(location);
        });

        array_texture.init_array(&mut self.images, &mut self.mats);

        self.commands.insert_resource(array_texture);
    }
}

#[derive(Debug, Default)]
struct TextureManagerInner {
    textures: BTreeMap<AssetLocation, TextureId>,

    dirty: bool,
}

#[derive(Resource, Debug)]
pub struct TextureManager<T: Asset + TypePath>(Arc<RwLock<TextureManagerInner>>, PhantomData<T>);

impl<T: Asset + TypePath> Default for TextureManager<T> {
    fn default() -> Self {
        Self(
            Arc::new(RwLock::new(TextureManagerInner {
                textures: BTreeMap::new(),
                dirty: false,
            })),
            PhantomData,
        )
    }
}

impl<T: Asset + TypePath> Clone for TextureManager<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }
}

impl<T: Asset + TypePath> TextureManager<T> {
    pub fn get_or_insert(&self, location: &AssetLocation) -> TextureId {
        {
            let map = self.0.read();
            if let Some(&id) = map.textures.get(location) {
                return id;
            }
        }

        let mut map = self.0.write();

        if let Some(&id) = map.textures.get(location) {
            return id;
        }

        let texture_id = TextureId(map.textures.len() as u32);

        map.textures.insert(location.clone(), texture_id);

        map.dirty = true;

        texture_id
    }
}
