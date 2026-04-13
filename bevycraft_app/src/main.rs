use bevy::prelude::*;
use bevycraft_app::AppState;
use bevycraft_render::prelude::ArrayTexture;
use bevycraft_world::prelude::{BlockDefinition, BlockFlags};

fn main() -> AppExit {
    App::new()
        .add_plugins((
            DefaultPlugins,
        ))
        .init_state::<AppState>()
        .insert_resource(Time::<Fixed>::from_hz(64.0))
        .run()
}