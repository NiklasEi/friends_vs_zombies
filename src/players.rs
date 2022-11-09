use crate::GameState;
use bevy::prelude::*;

pub struct PlayersPlugin;

impl Plugin for PlayersPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerColors>()
            .add_system_set(SystemSet::on_update(GameState::InGame).with_system(camera_follow));
    }
}

pub struct LocalPlayerId(pub usize);

#[derive(Component)]
pub struct Player {
    pub handle: usize,
}

pub struct PlayerColors(pub(crate) Vec<Color>);

impl Default for PlayerColors {
    fn default() -> Self {
        PlayerColors(vec![
            Color::rgba_u8(255, 0, 0, 255),
            Color::rgba_u8(0, 255, 0, 255),
            Color::rgba_u8(0, 0, 255, 255),
            Color::rgba_u8(0, 255, 255, 255),
            Color::rgba_u8(255, 0, 255, 255),
            Color::rgba_u8(255, 255, 0, 255),
        ])
    }
}

#[derive(Component, Reflect, Default)]
pub struct BulletReady(pub bool);

#[derive(Component, Reflect, Default, Clone, Copy)]
pub struct MoveDir(pub Vec2);

fn camera_follow(
    player_handle: Option<Res<LocalPlayerId>>,
    player_query: Query<(&Player, &Transform)>,
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Player>)>,
) {
    let player_handle = match player_handle {
        Some(handle) => handle.0,
        None => return, // Session hasn't started yet
    };

    for (player, player_transform) in player_query.iter() {
        if player.handle != player_handle {
            continue;
        }

        let pos = player_transform.translation;

        for mut transform in camera_query.iter_mut() {
            transform.translation.x = pos.x;
            transform.translation.y = pos.y;
        }
    }
}
