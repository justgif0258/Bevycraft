use bevy::prelude::*;
use bevycraft_world::prelude::SectionPool;

pub fn gc_task(
    mut pool    : ResMut<SectionPool>,
    mut timer   : Local<f32>,
    time        : Res<Time>
) {
    *timer += time.delta_secs();

    if *timer >= 10.0 {
        pool.collect_garbage(time.elapsed_secs_f64());
        
        *timer = 0.0;
    }
}