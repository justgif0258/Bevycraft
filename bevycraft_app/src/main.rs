use bevy::prelude::*;
use bevycraft_world::builtin::*;
use bevycraft_world::prelude::{BlockDefinition, BlockFlags};

fn main() -> AppExit {
    App::new()
        .add_plugins((
            DefaultPlugins,
        ))
        .insert_resource(Time::<Fixed>::from_hz(64.0))
        .add_systems(PreStartup, startup)
        .run()
}

fn startup() {
    let stone = BlockDefinition::new()
        .add(&HARDNESS, 2.0)
        .add(&TOUGHNESS, 6.0)
        .add(&FLAGS, BlockFlags::CAN_SUPPORT | BlockFlags::COLLIDABLE | BlockFlags::OCCLUDABLE)
        .build();

    let q = &FRICTION;

    let result = *stone.get(q);

    println!("Value of '{}': {:?}", q.name(), result);
}
