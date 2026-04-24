use bevy::prelude::{Assets, Commands, Mesh, Res, ResMut};
use bevycraft_render::renderer::level_renderer::LevelRenderer;
use bevycraft_world::prelude::{ChunkPos, Level};

pub fn render(
    mut commands: Commands,
    mut renderer: ResMut<LevelRenderer>,
    mut meshes  : ResMut<Assets<Mesh>>,
    level       : Res<Level>,
) {
    renderer.render_chunk(
        &mut commands,
        &mut meshes,
        &level,
        ChunkPos::new(0, 0)
    )
}