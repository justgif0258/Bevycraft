use bevy::prelude::*;
use bevycraft_app::prelude::*;
use bevycraft_core::prelude::{Namespace, Path, ResourceId};
use bevycraft_world::prelude::*;

const DEFAULT_NAMESPACE: Namespace = Namespace::new("bevycraft");

fn main() {
    let path = Path::new("stone");

    println!("{}", path.prefix("block/"));

    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(GameRegistries::default())
        .add_systems(Startup, init)
        .run();
}

fn init(root: Res<GameRegistries>) {
    let result = root.get_registered::<Block>(&ResourceId::parse("oak_leaves"));

    println!("{:#?}", result);
}
