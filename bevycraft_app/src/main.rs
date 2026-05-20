#[allow(unused_imports)]
#[cfg(debug_assertions)]
use bevy_dylib;
use {
    bevy::{
        anti_alias::fxaa::Fxaa,
        camera::Exposure,
        camera_controller::free_camera::{FreeCamera, FreeCameraPlugin},
        core_pipeline::tonemapping::Tonemapping,
        light::{
            light_consts::lux, AtmosphereEnvironmentMapLight, CascadeShadowConfigBuilder,
            DirectionalLightShadowMap, VolumetricFog, VolumetricLight,
        },
        pbr::{Atmosphere, AtmosphereMode, AtmosphereSettings, ScatteringMedium},
        post_process::bloom::Bloom,
        prelude::*,
        render::{
            settings::{Backends, RenderCreation, WgpuSettings},
            RenderPlugin,
        },
        tasks::available_parallelism,
    },
    bevycraft_app::*,
    bevycraft_core::prelude::*,
    bevycraft_render::prelude::*,
    bevycraft_world::prelude::*,
    ron::{extensions::Extensions, Options},
    std::f32::consts::FRAC_PI_8,
};

#[cfg(not(debug_assertions))]
#[global_allocator]
static ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;

const BLOCK_RES: u32 = 8;

fn main() -> AppExit {
    App::new()
        .add_plugins((
            DefaultPlugins.set(RenderPlugin {
                render_creation: RenderCreation::Automatic(WgpuSettings {
                    backends: Some(Backends::VULKAN),
                    ..default()
                }),
                ..default()
            }),
            FreeCameraPlugin,
            RModelPlugin::<BlockModel>::default(),
            MaterialPlugin::<VertexMaterial>::default(),
            ChunkPlugin::new(12, available_parallelism(), AppState::InGame),
        ))
        .init_state::<AppState>()
        .insert_resource(Time::<Fixed>::from_hz(64.0))
        .insert_resource(AssetsLoading::default())
        .add_systems(OnEnter(AppState::ModelDiscovery), discover_models)
        .add_systems(OnEnter(AppState::BuildArrayTexture), build_array_texture)
        .add_systems(
            FixedPostUpdate,
            await_models.run_if(in_state(AppState::AwaitModels)),
        )
        .add_systems(
            OnEnter(AppState::Finishing),
            (setup_world, view_loaded_models),
        )
        // .add_systems(FixedUpdate, (
        // ).run_if(in_state(AppState::InGame)))
        .run()
}

fn discover_models(
    mut commands: Commands,
    mut state: ResMut<NextState<AppState>>,
    server: Res<AssetServer>,
) {
    info!("Discovering models...");

    let blocks = Registrar::<Block>::read_from_registry();

    let mut manager = ModelManager::<Block, BlockModel>::with_capacity(blocks.len());

    blocks.iter().for_each(|(block_key, block)| {
        if block.air() {
            return;
        }

        let location = block_key.prefix("models/block/").suffix(".ron");

        let h = server.load_with_settings::<BlockModel, Options>(location, |options| {
            options
                .default_extensions
                .set(Extensions::IMPLICIT_SOME, true);
        });

        manager.set(blocks.key_to_idx(block_key).unwrap(), h);
    });

    commands.insert_resource(manager);

    state.set(AppState::AwaitModels);
}

fn await_models(
    mut commands: Commands,
    mut state: ResMut<NextState<AppState>>,
    manager: If<Res<ModelManager<Block, BlockModel>>>,
    server: Res<AssetServer>,
) {
    if manager.is_all_loaded(&server) {
        let id = commands.register_system(build_cache);

        commands.run_system(id);

        commands.unregister_system(id);

        state.set(AppState::BuildArrayTexture);
    }
}

fn build_cache(world: &mut World) {
    let manager = world
        .remove_resource::<ModelManager<Block, BlockModel>>()
        .unwrap();

    let mut assets = world.get_resource_mut::<Assets<BlockModel>>().unwrap();

    let cache = manager.build_cache(&mut assets);

    world.insert_resource(cache);

    info!("Successfully built model cache");
}

fn build_array_texture(
    mut state: ResMut<NextState<AppState>>,
    mut baker: TextureBakery<BlockModel>,
) {
    baker.bake(BLOCK_RES, BLOCK_RES);

    state.set(AppState::Finishing);
}

fn setup_world(
    mut commands: Commands,
    mut scattering: ResMut<Assets<ScatteringMedium>>,
    mut state: ResMut<NextState<AppState>>,
) {
    let medium = scattering.add(ScatteringMedium::earthlike(256, 256));

    commands.insert_resource(DirectionalLightShadowMap { size: 1024 });

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
            num_cascades: 3,
            maximum_distance: 128.0,
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
        ChunkLoader,
    ));

    commands.insert_resource::<GeneratorResource>(
        SimpleGenerator {
            seed: 0,
            amplitude_min: 0.0,
            amplitude_max: 192.0,
            octaves: 7,
            frequency: 0.03,
            gain: 1.0,
            lacunarity: 0.5,
        }
        .into(),
    );

    state.set(AppState::InGame);
}

fn view_loaded_models(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    cache: Res<ModelCache<Block, BlockModel>>,
    mats: Res<ArrayTexture>,
) {
    let mut buf = VertexBuffer::new();

    let tint = Some([0.2, 0.8, 0.2]);

    cache.iter().enumerate().for_each(|(i, model)| {
        if let Some(model) = model {
            let offset = [i as f32, 0.0, 0.0];

            for face in Direction::ALL {
                buf.push_quads_with_offset(model.iter_outer_quads_at(face), offset, tint);
            }

            buf.push_quads_with_offset(model.iter_inner_quads(), offset, tint);
        }
    });

    commands.spawn((
        Mesh3d(meshes.add(buf)),
        MeshMaterial3d(mats.get_vertex_material(RenderMode::Cutout)),
    ));

    let cube = meshes.add(Cuboid::new(2.0, 2.0, 2.0));

    let material = materials.add(StandardMaterial::default());

    commands.spawn((
        Mesh3d(cube.clone()),
        MeshMaterial3d(material.clone()),
        Transform::from_xyz(4.0, 1.0, 5.0)
    ));

    commands.spawn((
        Mesh3d(cube),
        MeshMaterial3d(material),
        Transform::from_xyz(4.5, 1.0, 7.5)
    ));
}
