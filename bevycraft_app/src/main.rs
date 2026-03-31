use bevy::prelude::*;
use bevycraft_world::{
    prelude::{Block, BlockBehaviour},
    presets::block::BlockFlags,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stone = BlockBehaviour::new()
        .hardness(2.0)
        .toughness(6.0)
        .flags(
            BlockFlags::COLLIDABLE
                | BlockFlags::OCCLUDABLE
                | BlockFlags::CAN_SUPPORT
                | BlockFlags::DOES_SPAWN,
        )
        .build();

    println!("{:?}", stone);

    /*
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(VirtualizedPool::<u16>::new(8192, 8192))
        .run();

     */

    Ok(())
}
