use crate::networking::{Dead, SeedFrame};
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
    pub last_attack: u32,
    pub attack_cooldown: u32,
}

pub fn kill_enemies(
    mut commands: Commands,
    mut enemy_query: Query<(Entity, &Transform, &mut Health), (With<Enemy>, Without<Bullet>)>,
    mut bullet_query: Query<(Entity, &Transform, &mut Bullet)>,
) {
    'bullets: for (bullet_entity, bullet_transform, mut bullet) in bullet_query.iter_mut() {
        for (enemy, enemy_transform, mut health) in enemy_query.iter_mut() {
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
            if bullet.is_used_up() {
                commands.entity(bullet_entity).despawn_recursive();
                continue 'bullets;
            }
        }
    }
}

pub fn move_enemies(
    mut enemy_query: Query<(&mut Transform, &mut Enemy)>,
    mut player_query: Query<
        (&Transform, &mut Health),
        (Without<Enemy>, With<Player>, Without<Dead>),
    >,
    seed_frame: Res<SeedFrame>,
) {
    for (mut transform, mut enemy) in &mut enemy_query {
        if let Some((closest_position, mut player_health)) =
            player_query.iter_mut().reduce(|closest, current| {
                if closest
                    .0
                    .translation
                    .distance_squared(transform.translation)
                    > current
                        .0
                        .translation
                        .distance_squared(transform.translation)
                {
                    current
                } else {
                    closest
                }
            })
        {
            let distance = closest_position.translation.xy() - transform.translation.xy();
            if distance.length() < PLAYER_RADIUS / 4. {
                if enemy.last_attack + enemy.attack_cooldown < seed_frame.0 {
                    enemy.last_attack = seed_frame.0;
                    player_health.current -= enemy.damage;
                }
                continue;
            }

            let move_delta = distance.normalize_or_zero() * enemy.speed;
            transform.translation.x += move_delta.x;
            transform.translation.y += move_delta.y;
        }
    }
}
