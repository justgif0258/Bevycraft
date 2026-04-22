use std::f32::consts::FRAC_PI_8;
use std::sync::Arc;
use bevy::anti_alias::fxaa::Fxaa;
use bevy::camera::Exposure;
use bevy::camera_controller::free_camera::{FreeCamera, FreeCameraPlugin};
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::light::*;
use bevy::light::light_consts::lux;
use bevy::pbr::*;
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevycraft_app::{AppState, GlobalRecords, Player, WorldRender};
use bevycraft_app::systems::chunking::{handle_chunk_tasks, pool_chunks, trash_chunks};
use bevycraft_app::systems::register::bootstrap_registries;
use bevycraft_core::prelude::*;
use bevycraft_render::prelude::*;
use bevycraft_world::prelude::*;

const BLOCK_RESOLUTION  : u32 = 8;

fn main() -> AppExit {
    println!("Sizeof BlockId: {}", size_of::<BlockType>());

    App::new()
        .add_plugins((
            DefaultPlugins,
            FreeCameraPlugin,
            MaterialPlugin::<VertexMaterial>::default(),
        ))
        .insert_resource(
            Time::<Fixed>::from_hz(64.0)
        )
        .init_state::<AppState>()
        .add_systems(OnEnter(AppState::LoadingContent), (
            bootstrap_registries,
            init
        ).chain())
        .add_systems(
            FixedUpdate,
            finish_loading_textures.run_if(in_state(AppState::WaitingForServer)),
        )
        .add_systems(OnEnter(AppState::BakingRenderers), bake_renderers)
        .add_systems(OnEnter(AppState::InGame), (
            setup_world,
        ).chain())
        .add_systems(FixedUpdate, (
            pool_chunks,
            handle_chunk_tasks,
            trash_chunks
        ).run_if(in_state(AppState::InGame)))
        .run()
}

fn init(
    mut commands: Commands,
    mut state: ResMut<NextState<AppState>>,
    global: Res<GlobalRecords>,
    asset_server: Res<AssetServer>,
) {
    info!("Initializing app...");
    info!("Compiling blocks to record...");

    info!("Loading block models...");

    let mut model_manager = RModelManager::default();

    global.blocks.iter_keys()
        .for_each(|block_key| {
            let path = block_key.prefix("block/");

            model_manager.load(path)
                .unwrap_or_else(|e| warn!("{}", e))
        });

    let mut textures_holder = TexturesHolder::default();

    model_manager.get_textures_locations()
        .iter()
        .for_each(|location| {
            let path = format!("{}/textures/{}.png", location.namespace(), location.path());

            textures_holder.storage.push((
                location.clone(),
                asset_server.load::<Image>(&path)
            ));
        });

    commands.insert_resource(textures_holder);
    commands.insert_resource(model_manager);

    state.set(AppState::WaitingForServer);
}

fn finish_loading_textures(
    mut state   : ResMut<NextState<AppState>>,
    asset_server: Res<AssetServer>,
    holder      : Res<TexturesHolder>,
) {
    if !holder.all_loaded(&asset_server) {
        return;
    }

    state.set(AppState::BakingRenderers);
}

fn bake_renderers(
    mut commands: Commands,
    mut state   : ResMut<NextState<AppState>>,
    mut mats    : ResMut<Assets<VertexMaterial>>,
    mut images  : ResMut<Assets<Image>>,
    manager     : Res<RModelManager>,
    holder      : Res<TexturesHolder>,
    global      : Res<GlobalRecords>,
) {
    let mut array_texture_builder = ArrayTexture::builder(BLOCK_RESOLUTION);

    holder.storage
        .iter()
        .for_each(|(location, handle)| {
            let image = images.remove(handle).unwrap();
            let data = image.data.unwrap();

            array_texture_builder.push(location.clone(), data);
        });

    commands.remove_resource::<TexturesHolder>();

    let array_texture = array_texture_builder.build_and_send(&mut mats, &mut images);

    info!("Successfully built array texture");

    let mut cache = BlockMeshCache::builder();

    global.blocks
        .iter_keys()
        .enumerate()
        .for_each(|(i, key)| {
            let model_key = key.prefix("block/");

            if let Some(model) = manager.get(&model_key) {
                match cache.bake_and_add_mesh(
                    &manager,
                    &array_texture,
                    model,
                    i
                ) {
                    Ok(_) => {},
                    Err(errors) => {
                        warn!("There were errors while loading mesh:");

                        errors.into_iter()
                            .for_each(|e| {
                                warn!(" |- {}", e);
                            })
                    },
                }
            }
        });

    commands.remove_resource::<RModelManager>();
    
    commands.insert_resource(WorldRender {
        meshes: Arc::new(cache.build()),
        materials: Arc::new(array_texture),
    });

    info!("Successfully baked block meshes");

    state.set(AppState::InGame);
}

fn setup_world(
    mut commands    : Commands,
    mut scattering  : ResMut<Assets<ScatteringMedium>>,
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
        }.build(),
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
        Player
    ));

    let manager = ChunkAccessor::new(
        8,
        BasicGenerator {
            seed: 5,
            frequency: 0.03,
            octaves: 7,
            amplitude_min: 0.0,
            amplitude_max: 128.0,
            min_height: 0,
            max_height: 256,
            snow_height: 112
        }
    );

    commands.insert_resource(manager);
}

#[derive(Resource, Default)]
pub struct TexturesHolder {
    pub storage: Vec<(AssetLocation, Handle<Image>)>,
}

impl TexturesHolder {
    #[inline]
    fn all_loaded(&self, asset_server: &AssetServer) -> bool {
        self.storage.iter().all(|(_, handle)| {
            asset_server.is_loaded_with_dependencies(handle)
        })
    }
}
