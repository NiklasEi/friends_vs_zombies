use crate::networking::{Dead, SeedFrame};
use crate::players::{Health, Player};
use crate::{Bullet, Score, BULLET_RADIUS, ENEMY_RADIUS, PLAYER_RADIUS};
use bevy::math::Vec3Swizzles;
use bevy::prelude::*;

pub struct EnemiesPlugin;

impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RollbackSafeEvents>();
    }
}

#[derive(Component)]
pub struct Enemy {
    pub damage: f64,
    pub speed: f32,
    pub last_attack: u32,
    pub attack_cooldown: u32,
}

pub struct SafeEvent {
    pub real_age: u32,
    pub id: u32,
    pub event: FvzEvent,
}

impl SafeEvent {
    pub fn new(event: FvzEvent, id: u32) -> Self {
        SafeEvent {
            real_age: 0,
            id,
            event,
        }
    }
}

pub enum FvzEvent {
    EnemyFall,
    PlayerHit,
    PlayerHitBullet,
    Lost,
    Pew,
    Revive,
}

#[derive(Default)]
pub struct RollbackSafeEvents(pub(crate) Vec<SafeEvent>);

pub fn kill_enemies(
    mut commands: Commands,
    mut score: ResMut<Score>,
    mut enemy_query: Query<(Entity, &Transform, &mut Health), (With<Enemy>, Without<Bullet>)>,
    mut bullet_query: Query<(Entity, &Transform, &mut Bullet)>,
    mut rollback_safe_events: ResMut<RollbackSafeEvents>,
) {
    'bullets: for (bullet_entity, bullet_transform, mut bullet) in bullet_query.iter_mut() {
        if bullet.is_used_up() {
            continue;
        }
        for (enemy, enemy_transform, mut health) in enemy_query.iter_mut() {
            let distance = Vec2::distance(
                enemy_transform.translation.xy(),
                bullet_transform.translation.xy(),
            );
            if distance < ENEMY_RADIUS + BULLET_RADIUS && bullet.hit(enemy) {
                score.0 += bullet.damage;
                health.current = (health.current - bullet.damage).max(0.);
                if health.current <= 0. {
                    rollback_safe_events.0.push(SafeEvent::new(
                        FvzEvent::EnemyFall,
                        enemy.id().wrapping_add(enemy.generation()),
                    ));
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
    mut enemy_query: Query<(Entity, &mut Transform, &mut Enemy)>,
    mut player_query: Query<
        (Entity, &Transform, &mut Health),
        (Without<Enemy>, With<Player>, Without<Dead>),
    >,
    seed_frame: Res<SeedFrame>,
    mut rollback_safe_events: ResMut<RollbackSafeEvents>,
) {
    for (enemy_entity, mut transform, mut enemy) in &mut enemy_query {
        if let Some((player, closest_position, mut player_health)) =
            player_query.iter_mut().reduce(|closest, current| {
                if closest
                    .1
                    .translation
                    .distance_squared(transform.translation)
                    > current
                        .1
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
                    rollback_safe_events.0.push(SafeEvent::new(
                        FvzEvent::PlayerHit,
                        (3 * player.id()).wrapping_add(enemy_entity.id()),
                    ));

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
