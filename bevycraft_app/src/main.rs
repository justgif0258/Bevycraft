use std::{f32::consts::FRAC_PI_8, sync::Arc};

use bevy::{
    anti_alias::fxaa::Fxaa,
    camera::Exposure,
    camera_controller::free_camera::{FreeCamera, FreeCameraPlugin},
    core_pipeline::tonemapping::Tonemapping,
    light::{
        AtmosphereEnvironmentMapLight, CascadeShadowConfigBuilder, VolumetricFog, VolumetricLight,
        light_consts::lux,
    },
    pbr::{Atmosphere, AtmosphereMode, AtmosphereSettings, ScatteringMedium},
    post_process::bloom::Bloom,
    prelude::*,
};
use bevycraft_app::{AppState, GlobalRecords, Player, systems::register::bootstrap_registries};
use bevycraft_core::prelude::{AssetLocation, Record};
use bevycraft_render::prelude::{
    ArrayTexture, BlockMeshCache, RModel, RModelManager, VertexMaterial,
};
use bevycraft_world::prelude::BlockType;

const BLOCK_RESOLUTION: u32 = 8;

fn main() -> AppExit {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FreeCameraPlugin,
            MaterialPlugin::<VertexMaterial>::default(),
        ))
        .insert_resource(Time::<Fixed>::from_hz(64.0))
        .init_state::<AppState>()
        .add_systems(
            OnEnter(AppState::LoadingContent),
            (bootstrap_registries, init).chain(),
        )
        .add_systems(OnEnter(AppState::BakingRenderers), bake_renderers)
        .add_systems(OnEnter(AppState::InGame), (setup_world,).chain())
        // .add_systems(FixedUpdate, (
        // ).run_if(in_state(AppState::InGame)))
        .run()
}

fn init(
    mut commands: Commands,
    mut state: ResMut<NextState<AppState>>,
    mut models: ResMut<Assets<RModel>>,
    mut images: ResMut<Assets<Image>>,
    mut mats: ResMut<Assets<VertexMaterial>>,
    global: Res<GlobalRecords>,
) {
    info!("Initializing app...");

    info!("Loading block models...");

    global.blocks.iter_keys().for_each(|block_key| {
        let path = format!(
            "{}/models/block/{}.ron",
            block_key.namespace(),
            block_key.path()
        );

        models.add(path);
    });

    state.set(AppState::WaitingForServer);
}

fn wait_models_to_load(
    mut state: ResMut<NextState<AppState>>,
    server: Res<AssetServer>,
    models: Res<Assets<RModel>>,
) {
    let ready = models
        .iter()
        .all(|(h, _): (Handle<RModel>, _)| server.is_loaded_with_dependencies(h));

    if ready {
        state.set(AppState::BakingRenderers);
    }
}

fn bake_renderers(
    mut commands: Commands,
    mut state: ResMut<NextState<AppState>>,
    textures: Res<ArrayTexture>,
    global: Res<GlobalRecords>,
) {
    let mut cache = BlockMeshCache::builder();

    global.blocks.iter_keys().enumerate().for_each(|(i, key)| {
        let model_key = key.prefix("block/");

        if let Some(model) = manager.get(&model_key) {
            match cache.bake_and_add_mesh(&manager, &textures, model, BlockType::from(i as u32 + 1))
            {
                Ok(_) => {}
                Err(errors) => {
                    warn!("There were errors while loading mesh:");

                    errors.into_iter().for_each(|e| {
                        warn!(" |- {}", e);
                    })
                }
            }
        }
    });

    let cache = Arc::new(cache.build());

    commands.remove_resource::<RModelManager>();

    info!("Successfully baked block meshes");

    state.set(AppState::InGame);
}

fn setup_world(
    mut commands: Commands,
    mut scattering: ResMut<Assets<ScatteringMedium>>,
    global: Res<GlobalRecords>,
) {
    let medium = scattering.add(ScatteringMedium::earthlike(256, 256));

    commands.insert_resource(GlobalAmbientLight {
        color: Color::WHITE,
        brightness: 1.0,
        ..default()
    });

    commands.spawn((
        DirectionalLight {
            illuminance: lux::RAW_SUNLIGHT,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_rotation_x(-FRAC_PI_8 / 2.0)),
        VolumetricLight,
        CascadeShadowConfigBuilder {
            num_cascades: 4,
            maximum_distance: 384.0,
            ..default()
        }
        .build(),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        Atmosphere::earthlike(medium),
        AtmosphereSettings {
            rendering_method: AtmosphereMode::LookupTexture,
            ..default()
        },
        AtmosphereEnvironmentMapLight::default(),
        Exposure { ev100: 13.0 },
        Tonemapping::AcesFitted,
        Bloom::NATURAL,
        VolumetricFog {
            ambient_intensity: 1.0,
            ..default()
        },
        Fxaa::default(),
        FreeCamera {
            sensitivity: 0.2,
            friction: 25.0,
            walk_speed: 3.0,
            run_speed: 9.0,
            ..default()
        },
        Player,
    ));
}

#[derive(Resource, Default)]
pub struct TexturesHolder {
    pub storage: Vec<(AssetLocation, Handle<Image>)>,
}

impl TexturesHolder {
    #[inline]
    fn all_loaded(&self, asset_server: &AssetServer) -> bool {
        self.storage
            .iter()
            .all(|(_, handle)| asset_server.is_loaded_with_dependencies(handle))
    }
}
