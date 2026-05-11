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
use bevycraft_core::prelude::{Block, Registrar, RegistrarOps, Registry};
use bevycraft_render::prelude::{ArrayTexture, BlockModel, Direction, RModelPlugin, TextureBakery, VertexMaterial};
use ron::Options;
use ron::extensions::Extensions;

const BLOCK_RES: u32 = 8;

fn main() -> AppExit {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FreeCameraPlugin,
            RModelPlugin::<BlockModel>::default(),
            MaterialPlugin::<VertexMaterial>::default(),
        ))
        .init_state::<AppState>()
        .insert_resource(Time::<Fixed>::from_hz(64.0))
        .insert_resource(AssetsLoading::<BlockModel>::default())
        .add_systems(OnEnter(AppState::ModelDiscovery), discover_models)
        .add_systems(OnEnter(AppState::BuildArrayTexture), build_array_texture)
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
    mut loading: ResMut<AssetsLoading<BlockModel>>,
    server: Res<AssetServer>,
) {
    info!("Discovering models...");

    Registrar::<Block>::read_from_registry()
        .iter()
        .for_each(|(block_key, block)| {
            if block.air() {
                return;
            }

            let location = block_key.prefix("models/block/").suffix(".ron");

            let h = server.load_with_settings::<BlockModel, Options>(location, |options| {
                options
                    .default_extensions
                    .set(Extensions::IMPLICIT_SOME, true);
            });

            loading.0.push(h);
        });

    state.set(AppState::AwaitModels);
}

fn await_models(
    mut state: ResMut<NextState<AppState>>,
    loading: Res<AssetsLoading<BlockModel>>,
    server: Res<AssetServer>,
) {
    let ready = loading
        .0
        .iter()
        .all(|h| server.is_loaded_with_dependencies(h));

    if ready {
        state.set(AppState::BuildArrayTexture);
    }
}

fn build_array_texture(
    mut state: ResMut<NextState<AppState>>,
    mut baker: TextureBakery<BlockModel>,
) {
    baker.bake(BLOCK_RES, BLOCK_RES);

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

fn spawn_test_model(
    mut commands: Commands,
    loading: Res<AssetsLoading<BlockModel>>,
    models: Res<Assets<BlockModel>>,
    textures: Res<ArrayTexture>,
) {
    let model = models.get(&loading.0[0]).unwrap();
    
    for d in 0..6 {
        let dir = Direction::from(d);
        
        let quad = model.iter_outer_quads_at(dir);
    }
}

#[derive(Resource)]
pub struct AssetsLoading<T: Asset>(Vec<Handle<T>>);

impl<T: Asset> Default for AssetsLoading<T> {
    fn default() -> Self {
        Self(Vec::new())
    }
}
