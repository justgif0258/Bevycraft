use std::{f32::consts::FRAC_PI_8, sync::LazyLock};

use bevy::{
    anti_alias::fxaa::Fxaa,
    camera::Exposure,
    camera_controller::free_camera::{FreeCamera, FreeCameraPlugin},
    core_pipeline::tonemapping::Tonemapping,
    light::{
        AtmosphereEnvironmentMapLight, CascadeShadowConfigBuilder, VolumetricFog, VolumetricLight,
        light_consts::lux,
    },
    math::bounding::Aabb3d,
    pbr::{Atmosphere, AtmosphereMode, AtmosphereSettings, ScatteringMedium},
    post_process::bloom::Bloom,
    prelude::*,
};
use bevycraft_app::{AppState, Player};
use bevycraft_core::{
    loc,
    prelude::{
        AssetLocation, Block, BlockBehaviour, BlockFlags, DefaultedRegistry, GameRegistries,
        Registry,
    },
};
use bevycraft_render::prelude::{ArrayTexture, RModel, RModelLoader, VertexMaterial};

const FULL_SHAPE: Aabb3d = Aabb3d {
    min: Vec3A::new(0.0, 0.0, 0.0),
    max: Vec3A::new(1.0, 1.0, 1.0),
};

const HALF_SHAPE: Aabb3d = Aabb3d {
    min: Vec3A::new(0.0, 0.0, 0.0),
    max: Vec3A::new(1.0, 0.5, 1.0),
};

const FULL_BLOCK: LazyLock<BlockFlags> = LazyLock::new(|| {
    BlockFlags::OCCLUDABLE
        | BlockFlags::COLLIDABLE
        | BlockFlags::DOES_SPAWN
        | BlockFlags::CAN_SUPPORT
});

const BLOCK_RESOLUTION: u32 = 8;

static TEST: AssetLocation = loc!(minecraft:dirt);

fn main() -> AppExit {
    let mut blocks = DefaultedRegistry::<Block>::new(
        AssetLocation::with_default_namespace("air"),
        Block::default(),
    );

    bootstrap_blocks(&mut blocks);

    GameRegistries::init_blocks(blocks);

    App::new()
        .add_plugins((
            DefaultPlugins,
            FreeCameraPlugin,
            MaterialPlugin::<VertexMaterial>::default(),
        ))
        .init_asset::<RModel>()
        .init_asset_loader::<RModelLoader>()
        .init_state::<AppState>()
        .init_resource::<AssetsLoading>()
        .insert_resource(Time::<Fixed>::from_hz(64.0))
        .insert_resource(ArrayTexture::new_uninit(BLOCK_RESOLUTION, BLOCK_RESOLUTION))
        .add_systems(OnEnter(AppState::ModelDiscovery), discover_models)
        .add_systems(OnEnter(AppState::TextureDiscovery), discover_textures)
        .add_systems(OnEnter(AppState::CachingMeshes), cache_meshes)
        .add_systems(
            FixedPostUpdate,
            wait_models_to_load.run_if(in_state(AppState::FinishingLoadingModels)),
        )
        .add_systems(OnEnter(AppState::InGame), setup_world)
        // .add_systems(FixedUpdate, (
        // ).run_if(in_state(AppState::InGame)))
        .run()
}

fn discover_models(
    mut state: ResMut<NextState<AppState>>,
    mut loading: ResMut<AssetsLoading>,
    server: Res<AssetServer>,
) {
    info!("Initializing app...");

    info!("Discovering models...");

    GameRegistries::blocks()
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

            loading.0.push(h.untyped());
        });

    state.set(AppState::FinishingLoadingModels);
}

fn wait_models_to_load(
    mut state: ResMut<NextState<AppState>>,
    loading: Res<AssetsLoading>,
    server: Res<AssetServer>,
) {
    let ready = loading
        .0
        .iter()
        .all(|h: &UntypedHandle| server.is_loaded_with_dependencies(h));

    if ready {
        state.set(AppState::CachingMeshes);
    }
}

fn discover_textures(
    mut state: ResMut<NextState<AppState>>,
    mut array_texture: ResMut<ArrayTexture>,
    mut images: ResMut<Assets<Image>>,
    mut mats: ResMut<Assets<VertexMaterial>>,
    models: Res<Assets<RModel>>,
) {
    for (_, model) in models.iter() {
        model.textures().for_each(|location| {
            array_texture.load_from_asset_location(location);
        });
    }

    array_texture.init_array(&mut images, &mut mats);

    state.set(AppState::CachingMeshes);
}

fn cache_meshes(
    mut state: ResMut<NextState<AppState>>,
    server: Res<AssetServer>,
    models: Res<Assets<RModel>>,
    textures: Res<ArrayTexture>,
) {
    GameRegistries::blocks().iter().for_each(|(key, block)| {
        if block.air() {
            return;
        }

        let path = format!("{}/models/block/{}.ron", key.namespace(), key.path());

        let model_handle = server.load::<RModel>(path);

        let model = models.get(&model_handle).expect("Failed to load RModel");
    });
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

#[derive(Resource, Default)]
pub struct AssetsLoading(Vec<UntypedHandle>);

fn bootstrap_blocks(registry: &mut impl Registry<Item = Block>) {
    register_block(
        registry,
        "grass",
        BlockBehaviour::new()
            .hardness(0.0)
            .toughness(0.0)
            .flags(BlockFlags::empty())
            .build(),
        [],
    );

    register_block(
        registry,
        "poppy",
        BlockBehaviour::new()
            .hardness(0.0)
            .toughness(0.0)
            .flags(BlockFlags::empty())
            .build(),
        [],
    );

    register_block(
        registry,
        "grass_block",
        BlockBehaviour::new()
            .hardness(0.65)
            .toughness(0.65)
            .flags(*FULL_BLOCK)
            .build(),
        [FULL_SHAPE],
    );

    register_block(
        registry,
        "dirt",
        BlockBehaviour::new()
            .hardness(0.5)
            .toughness(0.5)
            .flags(*FULL_BLOCK)
            .build(),
        [FULL_SHAPE],
    );

    register_block(
        registry,
        "sand",
        BlockBehaviour::new()
            .hardness(0.5)
            .toughness(0.5)
            .flags(*FULL_BLOCK)
            .build(),
        [FULL_SHAPE],
    );

    register_block(
        registry,
        "stone",
        BlockBehaviour::new()
            .hardness(2.0)
            .toughness(6.0)
            .flags(*FULL_BLOCK)
            .build(),
        [FULL_SHAPE],
    );

    register_block(
        registry,
        "bedrock",
        BlockBehaviour::new()
            .hardness(f32::INFINITY)
            .toughness(f32::INFINITY)
            .flags(BlockFlags::COLLIDABLE | BlockFlags::OCCLUDABLE | BlockFlags::CAN_SUPPORT)
            .build(),
        [FULL_SHAPE],
    );

    register_block(
        registry,
        "oak_log",
        BlockBehaviour::new()
            .hardness(2.0)
            .toughness(2.0)
            .flags(*FULL_BLOCK)
            .build(),
        [FULL_SHAPE],
    );

    register_block(
        registry,
        "oak_planks",
        BlockBehaviour::new()
            .hardness(2.0)
            .toughness(3.0)
            .flags(*FULL_BLOCK)
            .build(),
        [FULL_SHAPE],
    );

    register_block(
        registry,
        "oak_planks_slab",
        BlockBehaviour::new()
            .hardness(2.0)
            .toughness(3.0)
            .flags(BlockFlags::COLLIDABLE | BlockFlags::OCCLUDABLE | BlockFlags::CAN_SUPPORT)
            .build(),
        [HALF_SHAPE],
    );

    register_block(
        registry,
        "oak_planks_stair",
        BlockBehaviour::new()
            .hardness(2.0)
            .toughness(3.0)
            .flags(BlockFlags::COLLIDABLE | BlockFlags::OCCLUDABLE | BlockFlags::CAN_SUPPORT)
            .build(),
        [
            HALF_SHAPE,
            Aabb3d::from_min_max([0.0, 4.0, 0.0], [8.0, 8.0, 4.0]),
        ],
    );

    register_block(
        registry,
        "oak_trapdoor",
        BlockBehaviour::new()
            .hardness(2.0)
            .toughness(3.0)
            .flags(BlockFlags::COLLIDABLE | BlockFlags::OCCLUDABLE | BlockFlags::CAN_SUPPORT)
            .build(),
        [Aabb3d::from_min_max([0.0, 0.0, 0.0], [8.0, 2.0, 8.0])],
    );

    register_block(
        registry,
        "oak_leaves",
        BlockBehaviour::new()
            .hardness(0.2)
            .toughness(0.2)
            .flags(BlockFlags::COLLIDABLE | BlockFlags::CAN_SUPPORT)
            .build(),
        [FULL_SHAPE],
    );

    register_block(
        registry,
        "snow_block",
        BlockBehaviour::new()
            .hardness(0.2)
            .toughness(0.2)
            .flags(*FULL_BLOCK)
            .build(),
        [FULL_SHAPE],
    );
}

fn register_block(
    registry: &mut impl Registry<Item = Block>,
    name: &'static str,
    behaviour: BlockBehaviour,
    shapes: impl Into<Box<[Aabb3d]>>,
) {
    registry
        .register(
            AssetLocation::with_default_namespace(name),
            Block::new().behaviour(behaviour).shapes(shapes).build(),
        )
        .expect("Registration failed");
}
