use crate::players::{Health, Player};
use crate::{Bullet, BULLET_RADIUS, ENEMY_RADIUS, PLAYER_RADIUS};
use bevy::math::Vec3Swizzles;
use bevy::prelude::*;

pub struct EnemiesPlugin;

impl Plugin for EnemiesPlugin {
    fn build(&self, _app: &mut App) {}
}

#[derive(Component)]
pub struct Enemy {
    pub damage: f64,
    pub speed: f32,
}

pub fn kill_enemies(
    mut commands: Commands,
    mut enemy_query: Query<(Entity, &Transform, &mut Health), (With<Enemy>, Without<Bullet>)>,
    mut bullet_query: Query<(Entity, &Transform, &mut Bullet)>,
) {
    for (enemy, enemy_transform, mut health) in enemy_query.iter_mut() {
        // Todo: despawn bullets
        for (_bullet_entity, bullet_transform, mut bullet) in bullet_query.iter_mut() {
            let distance = Vec2::distance(
                enemy_transform.translation.xy(),
                bullet_transform.translation.xy(),
            );
            if distance < ENEMY_RADIUS + BULLET_RADIUS && bullet.hit(enemy) {
                health.current = (health.current - bullet.damage).max(0.);
                if health.current <= 0. {
                    commands.entity(enemy).despawn_recursive();
                }
            }
        }
    }
}

pub fn move_enemies(
    mut enemy_query: Query<(&mut Transform, &Enemy)>,
    player_query: Query<&Transform, (Without<Enemy>, With<Player>)>,
) {
    for (mut transform, enemy) in &mut enemy_query {
        if let Some(closest_position) = player_query.iter().reduce(|closest, current| {
            if closest.translation.distance_squared(transform.translation)
                > current.translation.distance_squared(transform.translation)
            {
                current
            } else {
                closest
            }
        }) {
            let distance = closest_position.translation.xy() - transform.translation.xy();
            if distance.length() < PLAYER_RADIUS / 4. {
                continue;
            }

            let move_delta = distance.normalize_or_zero() * enemy.speed;
            transform.translation.x += move_delta.x;
            transform.translation.y += move_delta.y;
        }
    }
}
