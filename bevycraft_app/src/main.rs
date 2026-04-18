use std::f32::consts::FRAC_PI_8;
use bevy::anti_alias::fxaa::Fxaa;
use bevy::camera::Exposure;
use bevy::camera_controller::free_camera::{FreeCamera, FreeCameraPlugin};
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::light::*;
use bevy::light::light_consts::lux;
use bevy::pbr::*;
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevycraft_app::AppState;
use bevycraft_app::systems::register::register_blocks;
use bevycraft_core::prelude::*;
use bevycraft_render::prelude::*;
use bevycraft_world::prelude::*;

const MAX_GARBAGE_DELTA : f64 = 60.0f64;

const BLOCK_RESOLUTION  : u32 = 8;

fn main() -> AppExit {
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
        .add_systems(OnEnter(AppState::LoadingContent), init)
        .add_systems(
            FixedUpdate,
            finish_loading_textures.run_if(in_state(AppState::WaitingForServer)),
        )
        .add_systems(OnEnter(AppState::BakingRenderers), bake_renderers)
        .add_systems(OnEnter(AppState::InGame), (
            setup_world,
            setup_player,
        ).chain())
        .run()
}

fn init(
    mut commands: Commands,
    mut state   : ResMut<NextState<AppState>>,
    asset_server: Res<AssetServer>,
) {
    info!("Initializing app...");
    info!("Compiling blocks to record...");

    let blocks = register_blocks();

    info!("Loading block models...");

    let mut manager = RModelManager::default();

    blocks.keys()
        .iter()
        .for_each(|&block_key|
            manager.load(block_key.prefix("block/"))
                .unwrap_or_else(|e| warn!("{}", e))
        );

    let mut holder = TexturesHolder::default();

    manager.get_textures_locations()
        .iter()
        .for_each(|location| {
            let path = format!("{}/textures/{}.png", location.namespace(), location.path());

            holder.storage.push((
                location.clone(),
                asset_server.load::<Image>(&path)
            ));
        });

    commands.insert_resource(blocks);
    commands.insert_resource(holder);
    commands.insert_resource(manager);

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
    mut images  : ResMut<Assets<Image>>,
    mut manager : ResMut<RModelManager>,
    holder      : Res<TexturesHolder>,
    blocks      : Res<BlockRecord>,
) {
    let mut builder = ArrayTexture::builder(BLOCK_RESOLUTION);

    holder.storage
        .iter()
        .for_each(|(location, handle)| {
            let image = images.remove(handle).unwrap();
            let data = image.data.unwrap();

            builder.register(location.clone(), data);
        });

    commands.remove_resource::<TexturesHolder>();

    let array_texture = builder.build_and_send(&mut images);

    info!("Successfully built array texture");

    let mut mesh_manager = BlockMeshManager::builder();

    blocks.iter_definitions()
        .enumerate()
        .for_each(|(i, def)| {
            let mut flags = RenderFlags::empty();

            if def.translucent() {
                flags |= RenderFlags::TRANSLUCENT;
            } else {
                flags |= RenderFlags::OPAQUE;
            }

            if def.greedy_meshable() {
                flags |= RenderFlags::GREEDY_MESHABLE;
            }

            if def.occludable() {
                flags |= RenderFlags::OCCLUDABLE;
            }

            let model_key = blocks.idx_to_key(i)
                .unwrap()
                .prefix("block/");

            if let Some(model) = manager.take(&model_key) {
                match mesh_manager.bake_and_add_mesh(
                    &manager,
                    &array_texture,
                    model,
                    flags,
                    i as u32
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

    commands.insert_resource(mesh_manager.build());

    commands.insert_resource(array_texture);


    info!("Successfully baked block meshes");

    state.set(AppState::InGame);
}

fn setup_world(
    mut commands    : Commands,
    mut scattering  : ResMut<Assets<ScatteringMedium>>,
) {
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
            first_cascade_far_bound: 0.3,
            maximum_distance: 15.0,
            ..default()
        }.build(),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        Atmosphere::earthlike(scattering.add(ScatteringMedium::default())),
        AtmosphereSettings::default(),
        Exposure { ev100: 13.0 },
        Tonemapping::AcesFitted,
        Bloom::NATURAL,
        AtmosphereEnvironmentMapLight::default(),
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

    let generator = ActiveWorldGenerator::new(
        BasicGenerator {
            seed: rand::random(),
            frequency: 0.06,
            octaves: 4,
            amplitude_min: 0.0,
            amplitude_max: 24.0,
        }
    );

    commands.insert_resource(generator);
}

fn setup_player(
    mut commands: Commands,
    mut meshes  : ResMut<Assets<Mesh>>,
    mut vertex  : ResMut<Assets<VertexMaterial>>,
    blocks      : Res<BlockRecord>,
    block_meshes: Res<BlockMeshManager>,
    array_texture: Res<ArrayTexture>,
    generator   : Res<ActiveWorldGenerator>,
) {
    let mut chunks = Vec::new();

    for x in 0..8 {
        for z in 0..8 {
            let chunk = Chunk::generate_using(
                [x, z],
                &blocks,
                generator.generator.as_ref(),
            );

            chunks.push(chunk);
        }
    }

    let material = array_texture.get_vertex_material(&mut vertex);

    for chunk in chunks {
        let mesh = meshes.add(chunk.build_chunk_mesh(&block_meshes));

        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material.clone()),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));
    }
}

#[derive(Component)]
pub struct Player;

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
