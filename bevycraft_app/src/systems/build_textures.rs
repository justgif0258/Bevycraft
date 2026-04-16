use bevy::prelude::*;
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevycraft_core::prelude::AssetLocation;
use bevycraft_render::prelude::{ArrayTexture, RModelManager, VertexMaterial};
use crate::AppState;

pub fn load_textures_into_server(
    mut commands: Commands,
    manager: Res<RModelManager>,
    asset_server: Res<AssetServer>,
) {
    let mut handles = TextureHandles::default();

    manager.get_textures_locations()
        .iter()
        .for_each(|location| {
            let handle = asset_server.load::<Image>(
                format!("{}/textures/{}.png", location.namespace(), location.path())
            );

            handles.storage.push((location.clone(), handle));
        });

    commands.insert_resource(handles);

    info!("Waiting for server to finish loading textures...")
}

pub fn wait_for_server(
    handles: Res<TextureHandles>,
    asset_server: Res<AssetServer>,
    mut next: ResMut<NextState<AppState>>,
) {
    let all_loaded = handles.all_loaded(asset_server);

    if all_loaded {
        info!("Server finished loading textures. Beginning building array texture...");

        next.set(AppState::BuildingArrayTexture)
    }
}

pub fn build_array_texture(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut next: ResMut<NextState<AppState>>,
    manager: Res<RModelManager>,
    asset_server: Res<AssetServer>,
) {
    let mut holder: Vec<(AssetLocation, Handle<Image>)> = Vec::new();

    manager.get_textures_locations()
        .iter()
        .for_each(|location| {
            let handle = asset_server.load::<Image>(
                format!("{}/textures/{}.png", location.namespace(), location.path())
            );

            holder.push((location.clone(), handle));
        });

    let mut builder = ArrayTexture::builder(8);

    holder.into_iter()
        .for_each(|(location, handle)| {
            let image = images.get(&handle).unwrap();
            let data = image.data.clone().unwrap();

            builder.register(location, data);
        });

    commands.insert_resource(builder.build_and_send(&mut images));
    
    info!("Successfully built array texture");
    
    next.set(AppState::SolvingBlockModels);
}

#[derive(Resource)]
pub struct TextureHandles {
    pub storage: Vec<(AssetLocation, Handle<Image>)>,
}

impl Default for TextureHandles {
    #[inline]
    fn default() -> Self {
        Self { storage: vec![] }
    }
}

impl TextureHandles {
    #[inline]
    fn all_loaded(&self, server: Res<AssetServer>) -> bool {
        self.storage.iter().all(|(_, handle)| {
            server.is_loaded_with_dependencies(handle)
        })
    }
}