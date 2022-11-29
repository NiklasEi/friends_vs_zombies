use crate::networking::SeedFrame;
use crate::GameState;
use bevy::prelude::*;

pub struct PlayersPlugin;

impl Plugin for PlayersPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(GameState::InGame)
                .with_system(camera_follow)
                .with_system(animate_sprites),
        );
    }
}

pub struct LocalPlayerId(pub usize);

#[derive(Component)]
pub struct Player {
    pub handle: usize,
}

#[derive(Component)]
pub struct Health {
    pub max: f64,
    pub current: f64,
}

impl Health {
    pub(crate) fn new(hp: f64) -> Self {
        Health {
            max: hp,
            current: hp,
        }
    }
}

#[derive(Component, Reflect, Default)]
pub struct Weapon {
    fire_frame: u32,
    frame_cooldown: u32,
}

impl Weapon {
    pub fn new() -> Self {
        Weapon {
            fire_frame: 0,
            frame_cooldown: 30,
        }
    }

    pub fn shoot(&mut self, seed_frame: &SeedFrame) -> bool {
        if self.fire_frame.wrapping_add(self.frame_cooldown) < seed_frame.0 {
            self.fire_frame = seed_frame.0;
            true
        } else {
            false
        }
    }
}

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

#[derive(Component, Reflect, Default)]
pub struct AnimationTimer(pub Timer, pub usize);

fn animate_sprites(
    time: Res<Time>,
    mut query: Query<(&mut AnimationTimer, &mut TextureAtlasSprite)>,
) {
    for (mut timer, mut sprite) in &mut query {
        if timer.0.paused() {
            sprite.index = 0;
            continue;
        }
        timer.0.tick(time.delta());
        if timer.0.finished() {
            sprite.index = (sprite.index + 1) % timer.1;
        }
    }
}
