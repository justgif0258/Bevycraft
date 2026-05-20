use {
    crate::prelude::{
        ArrayTexture, Model, TextureId, VertexMaterial, NULL_TEXTURE_ID, NULL_TEXTURE_LOCATION,
    },
    bevy::{
        ecs::{resource::Resource, system::SystemParam},
        prelude::{error, Asset, Assets, Commands, Image, Res, ResMut, TypePath},
    },
    bevycraft_core::prelude::AssetLocation,
    parking_lot::RwLock,
    std::{marker::PhantomData, path::PathBuf, sync::Arc},
};

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
    textures: Vec<(AssetLocation, TextureId)>,

    dirty: bool,
}

#[derive(Resource, Debug)]
pub struct TextureManager<T: Asset + TypePath>(Arc<RwLock<TextureManagerInner>>, PhantomData<T>);

impl<T: Asset + TypePath> Default for TextureManager<T> {
    fn default() -> Self {
        let mut textures = Vec::new();

        textures.push((NULL_TEXTURE_LOCATION.clone(), NULL_TEXTURE_ID));

        Self(
            Arc::new(RwLock::new(TextureManagerInner {
                textures,
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

            if let Some(&(_, id)) = map.textures.iter().find(|&(loc, _)| loc == location) {
                return id;
            }
        }

        let mut map = self.0.write();

        if let Some(&(_, id)) = map.textures.iter().find(|&(loc, _)| loc == location) {
            return id;
        }

        let path: PathBuf = PathBuf::from(format!(
            "bevycraft_app/assets/{}/textures/{}.png",
            location.namespace(),
            location.path()
        ));

        if path.exists() {
            let texture_id = TextureId(map.textures.len() as u32);

            map.textures.push((location.clone(), texture_id));

            map.dirty = true;

            texture_id
        } else {
            error!("Unable to find texture at: {}", location);

            NULL_TEXTURE_ID
        }
    }
}
