use bevy::prelude::*;
use bevycraft_app::AppState;
use bevycraft_app::systems::collector::gc_task;
use bevycraft_app::systems::register::register_blocks;
use bevycraft_core::prelude::*;
use bevycraft_render::prelude::*;
use bevycraft_world::prelude::*;

const MAX_GARBAGE_DELTA : f64 = 60.0f64;

const BLOCK_RESOLUTION  : u32 = 8;

fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(
            Time::<Fixed>::from_hz(64.0)
        )
        .insert_resource(
            SectionPool::new(MAX_GARBAGE_DELTA)
        )
        .init_state::<AppState>()
        .add_systems(OnEnter(AppState::LoadingContent), init)
        .add_systems(
            FixedUpdate,
            finish_loading_textures.run_if(in_state(AppState::WaitingForServer)),
        )
        .add_systems(OnEnter(AppState::BakingRenderers), bake_renderers)
        .add_systems(FixedUpdate, (
            gc_task,
            )
            .run_if(in_state(AppState::InGame))
        )
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

    let mut manager_builder = BlockMeshManager::builder();

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
                match manager_builder.bake_and_add_mesh(
                    &manager,
                    &array_texture,
                    model,
                    flags,
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

    commands.insert_resource(manager_builder.build());

    info!("Successfully baked block meshes");

    state.set(AppState::InGame);
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
