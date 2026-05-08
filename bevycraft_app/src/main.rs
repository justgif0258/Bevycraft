use std::f32::consts::FRAC_PI_8;

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
use bevycraft_app::{AppState, Player};
use bevycraft_core::prelude::{AssetLocation, Block, Registrar, Registry};
use bevycraft_render::prelude::{ArrayTexture, RModel, RModelPlugin, VertexMaterial};

const BLOCK_RES: u32 = 8;

fn main() -> AppExit {
    let blocks = Block::read_from_registry();

    blocks.iter().for_each(|(loc, _)| {
        println!("Registered {}", loc);
    });

    App::new()
        .add_plugins((
            DefaultPlugins,
            FreeCameraPlugin,
            RModelPlugin,
            MaterialPlugin::<VertexMaterial>::default(),
        ))
        .init_state::<AppState>()
        .init_resource::<AssetsLoading<RModel>>()
        .insert_resource(Time::<Fixed>::from_hz(64.0))
        .add_systems(OnEnter(AppState::ModelDiscovery), discover_models)
        .add_systems(OnEnter(AppState::BuildArrayTexture), build_array_texture)
        .add_systems(OnEnter(AppState::CacheMeshes), cache_meshes)
        .add_systems(
            FixedPostUpdate,
            await_models.run_if(in_state(AppState::AwaitModels)),
        )
        .add_systems(OnEnter(AppState::Finishing), setup_world)
        // .add_systems(FixedUpdate, (
        // ).run_if(in_state(AppState::InGame)))
        .run()
}

fn discover_models(
    mut state: ResMut<NextState<AppState>>,
    mut loading: ResMut<AssetsLoading<RModel>>,
    server: Res<AssetServer>,
) {
    info!("Discovering models...");

    Block::read_from_registry()
        .iter()
        .for_each(|(block_key, block)| {
            if block.air() {
                return;
            }

            let path = format!(
                "{}/models/block/{}.ron",
                block_key.namespace(),
                block_key.path()
            );

            let h = server.load::<RModel>(path);

            loading.0.push((block_key.clone(), h));
        });

    state.set(AppState::AwaitModels);
}

fn await_models(
    mut state: ResMut<NextState<AppState>>,
    loading: Res<AssetsLoading<RModel>>,
    server: Res<AssetServer>,
) {
    let ready = loading
        .0
        .iter()
        .all(|(_, h)| server.is_loaded_with_dependencies(h));

    if ready {
        state.set(AppState::BuildArrayTexture);
    }
}

fn build_array_texture(
    mut commands: Commands,
    mut state: ResMut<NextState<AppState>>,
    mut images: ResMut<Assets<Image>>,
    mut mats: ResMut<Assets<VertexMaterial>>,
    models: Res<Assets<RModel>>,
) {
    let mut array_texture = ArrayTexture::new_uninit(BLOCK_RES, BLOCK_RES);

    for (_, model) in models.iter() {
        model.textures().for_each(|location| {
            array_texture.load_from_asset_location(location);
        });
    }

    array_texture.init_array(&mut images, &mut mats);

    commands.insert_resource(array_texture);

    state.set(AppState::CacheMeshes);
}

fn cache_meshes(
    mut state: ResMut<NextState<AppState>>,
    server: Res<AssetServer>,
    models: Res<Assets<RModel>>,
    textures: Res<ArrayTexture>,
) {
    Block::read_from_registry().iter().for_each(|(key, block)| {
        if block.air() {
            return;
        }

        let path = format!("{}/models/block/{}.ron", key.namespace(), key.path());

        let model_handle = server.load::<RModel>(path);

        let model = models.get(&model_handle).expect("Failed to load RModel");
    });

    state.set(AppState::Finishing);
}

fn setup_world(mut commands: Commands, mut scattering: ResMut<Assets<ScatteringMedium>>) {
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

#[derive(Resource)]
pub struct AssetsLoading<T: Asset>(Vec<(AssetLocation, Handle<T>)>);

impl<T: Asset> Default for AssetsLoading<T> {
    fn default() -> Self {
        Self(Vec::new())
    }
}
