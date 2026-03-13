use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;
use bevy::prelude::*;
use bevycraft_core::prelude::{VirtualizedPool};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let u = AtomicUsize::new(0);

    u.fetch_sub(1, Relaxed);

    println!("{:?}", u);

    /*
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(VirtualizedPool::<u16>::new(8192, 8192))
        .run();

     */

    Ok(())
}