use bevy::prelude::*;
use bevycraft_world::prelude::ChunkMap;

#[derive(Component)]
struct ChunkCountText;

fn spawn_hud(mut commands: Commands) {
    commands.spawn((
        ChunkCountText,
        Text::new("Chunks: 0"),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(8.0),
            left: Val::Px(8.0),
            ..default()
        },
    ));
}

fn update_hud(chunk_map: Res<ChunkMap>, mut query: Query<&mut Text, With<ChunkCountText>>) {
    if !chunk_map.is_changed() {
        return;
    }

    let Ok(mut text) = query.single_mut() else {
        return;
    };

    text.0 = format!(
        "Chunks: {} loaded  |  {} pending  |  {} queued",
        chunk_map.loaded_count(),
        chunk_map.pending_count(),
        chunk_map.enqueued(),
    );
}

pub struct DebugHudPlugin;

impl Plugin for DebugHudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_hud)
            .add_systems(Update, update_hud);
    }
}
