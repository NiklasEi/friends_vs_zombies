use crate::enemies::{kill_enemies, move_enemies, Enemy};
use crate::loading::{EnemyAssets, EnemyData, PlayerAssets};
use crate::matchmaking::Seed;
use crate::players::{AnimationTimer, Health};
use crate::{
    direction, fire, game_input, Bullet, GameState, ImageAssets, MoveDir, Player, Weapon,
    BULLET_RADIUS, MAP_SIZE, PLAYER_RADIUS,
};
use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy_ggrs::{GGRSPlugin, Rollback, RollbackIdProvider};
use ggrs::{InputStatus, P2PSession};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::f32::consts::PI;
use std::time::Duration;

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EnemyTimer>()
            .init_resource::<SeedFrame>();
        GGRSPlugin::<GgrsConfig>::new()
            .with_input_system(game_input)
            .with_rollback_schedule(
                Schedule::default().with_stage(
                    "ROLLBACK_STAGE",
                    SystemStage::single_threaded()
                        .with_system_set(State::<GameState>::get_driver())
                        .with_system_set(
                            SystemSet::on_enter(GameState::Interlude)
                                .with_system(reset_interlude_timer),
                        )
                        .with_system_set(
                            SystemSet::on_exit(GameState::Interlude).with_system(remove_players),
                        )
                        .with_system_set(
                            SystemSet::on_update(GameState::Interlude).with_system(interlude_timer),
                        )
                        .with_system_set(
                            SystemSet::on_enter(GameState::InGame).with_system(spawn_players),
                        )
                        .with_system_set(
                            SystemSet::on_update(GameState::InGame)
                                .with_system(advance_seed_frame)
                                .with_system(spawn_enemies.after(advance_seed_frame))
                                .with_system(move_players.after(advance_seed_frame))
                                .with_system(move_bullet.after(advance_seed_frame))
                                .with_system(kill_enemies.after(move_bullet))
                                .with_system(fire_bullets.after(move_players))
                                .with_system(move_enemies.after(move_players))
                                .with_system(kill_players.after(kill_enemies).after(move_players)),
                        ),
                ),
            )
            .register_rollback_type::<Transform>()
            .register_rollback_type::<SeedFrame>()
            .register_rollback_type::<Weapon>()
            .register_rollback_type::<Bullet>()
            .register_rollback_type::<MoveDir>()
            .register_rollback_type::<EnemyTimer>()
            .register_rollback_type::<Dead>()
            .build(app);
    }
}

pub struct GgrsConfig;

impl ggrs::Config for GgrsConfig {
    // 4-directions + fire fits easily in a single byte
    type Input = u8;
    type State = u8;
    // Matchbox' WebRtcSocket addresses are strings
    type Address = String;
}

#[derive(Default)]
pub struct InterludeTimer(pub usize);

fn reset_interlude_timer(mut timer: ResMut<InterludeTimer>) {
    timer.0 = 60 * 3;
}

fn interlude_timer(mut timer: ResMut<InterludeTimer>, mut state: ResMut<State<GameState>>) {
    if timer.0 == 0 {
        state.set(GameState::InGame).unwrap();
    } else {
        timer.0 -= 1;
    }
}

#[derive(Component)]
pub struct HealthBar;

pub fn spawn_players(
    mut commands: Commands,
    mut rollback_id_provider: ResMut<RollbackIdProvider>,
    player_assets: Res<PlayerAssets>,
    session: Res<P2PSession<GgrsConfig>>,
) {
    info!("Spawning players");

    for player in 0..session.num_players() {
        commands
            .spawn_bundle(SpriteSheetBundle {
                transform: Transform {
                    translation: Vec3::new(0., 0., 100.),
                    scale: Vec3::splat(0.01),
                    ..Default::default()
                },
                sprite: TextureAtlasSprite::new(0),
                texture_atlas: player_assets.get_atlas(player).clone(),
                ..Default::default()
            })
            .insert(AnimationTimer(Timer::from_seconds(0.1, true), 4))
            .insert(Player { handle: player })
            .insert(Weapon::new())
            .insert(MoveDir(-Vec2::X))
            .insert(Health::new(500.))
            .insert(Rollback::new(rollback_id_provider.next_id()))
            .with_children(|parent| {
                parent.spawn_bundle(SpriteBundle {
                    sprite: Sprite {
                        color: Color::DARK_GRAY,
                        custom_size: Some(Vec2::new(100., 5.1)),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(0., 50., 1.)),
                    ..default()
                });
                parent
                    .spawn_bundle(SpriteBundle {
                        sprite: Sprite {
                            color: Color::RED,
                            custom_size: Some(Vec2::new(100., 5.1)),
                            ..default()
                        },
                        transform: Transform::from_translation(Vec3::new(0., 50., 2.)),
                        ..default()
                    })
                    .insert(HealthBar);
            });
    }
}

fn remove_players(
    mut commands: Commands,
    player_query: Query<Entity, With<Player>>,
    bullet_query: Query<Entity, With<Bullet>>,
) {
    for player in player_query.iter() {
        commands.entity(player).despawn_recursive();
    }
    for bullet in bullet_query.iter() {
        commands.entity(bullet).despawn_recursive();
    }
}

fn kill_players(
    mut commands: Commands,
    mut player_query: Query<
        (Entity, &mut Transform, &mut Health),
        (With<Player>, Without<Bullet>, Without<Dead>),
    >,
    mut bullet_query: Query<(&Transform, &mut Bullet)>,
) {
    for (player, mut player_transform, mut health) in player_query.iter_mut() {
        for (bullet_transform, mut bullet) in bullet_query.iter_mut() {
            let distance = Vec2::distance(
                player_transform.translation.xy(),
                bullet_transform.translation.xy(),
            );
            if distance < PLAYER_RADIUS + BULLET_RADIUS && bullet.hit(player) {
                health.current = (health.current - bullet.damage).max(0.);
                if health.current <= 0. {
                    commands.entity(player).insert(Dead);
                    player_transform.rotation = Quat::from_rotation_z(PI / 2.);
                }
            }
        }
    }
}

#[derive(Reflect, Component, Default)]
pub struct Dead;

fn move_players(
    inputs: Res<Vec<(u8, InputStatus)>>,
    mut player_query: Query<(
        Entity,
        &mut Transform,
        &mut MoveDir,
        &Player,
        &mut AnimationTimer,
    )>,
    dead: Query<&Dead>,
) {
    for (player_entity, mut transform, mut move_direction, player, mut animation_timer) in
        player_query.iter_mut()
    {
        if dead.contains(player_entity) {
            animation_timer.0.pause();
            continue;
        }
        let (input, _) = inputs[player.handle];
        let direction = direction(input);

        if direction == Vec2::ZERO {
            animation_timer.0.pause();
            continue;
        }
        if animation_timer.0.paused() {
            animation_timer.0.unpause();
        }

        move_direction.0 = direction;

        let move_speed = 0.13;
        let move_delta = direction * move_speed;

        let old_pos = transform.translation.xy();
        let limit = Vec2::splat(MAP_SIZE as f32 / 2. - 0.5);
        let new_pos = (old_pos + move_delta).clamp(-limit, limit);

        transform.translation.x = new_pos.x;
        transform.translation.y = new_pos.y;
    }
}

fn advance_seed_frame(mut frame: ResMut<SeedFrame>) {
    frame.0 = frame.0.wrapping_add(1);
}

#[derive(Reflect, Component, Default)]
pub struct SeedFrame(pub(crate) u32);

#[derive(Reflect, Component)]
struct EnemyTimer(Timer);

impl Default for EnemyTimer {
    fn default() -> Self {
        Self(Timer::new(Duration::from_secs_f32(5.), true))
    }
}

fn spawn_enemies(
    mut commands: Commands,
    seed: Res<Seed>,
    enemy_assets: Res<EnemyAssets>,
    enemy_data: Res<Assets<EnemyData>>,
    seed_frame: Res<SeedFrame>,
    mut enemy_timer: ResMut<EnemyTimer>,
    mut rollback_id_provider: ResMut<RollbackIdProvider>,
) {
    enemy_timer.0.tick(Duration::from_secs_f32(1. / 60.));
    if !enemy_timer.0.just_finished() {
        return;
    }
    let seed: [u8; 32] = [
        (seed_frame.0 >> 8) as u8,
        seed_frame.0 as u8,
        seed.0[1],
        seed.0[2],
        (seed_frame.0 >> 16) as u8,
        (seed_frame.0 >> 24) as u8,
        seed.0[1],
        seed.0[2],
    ]
    .repeat(4)
    .try_into()
    .unwrap();
    let mut rng = ChaCha8Rng::from_seed(seed);
    let translation = Vec3::new(
        rng.gen_range(0..MAP_SIZE) as f32 - MAP_SIZE as f32 / 2.,
        rng.gen_range(0..MAP_SIZE) as f32 - MAP_SIZE as f32 / 2.,
        100.,
    );
    let enemy_index = rng.gen_range(0..100);
    info!("Spawning enemy at {:?}", translation);
    spawn_enemy(
        &mut commands,
        &enemy_assets,
        &mut rollback_id_provider,
        &enemy_data,
        translation,
        enemy_index,
    );
}

fn spawn_enemy(
    commands: &mut Commands,
    enemy_assets: &EnemyAssets,
    rollback_id_provider: &mut RollbackIdProvider,
    enemy_data: &Assets<EnemyData>,
    translation: Vec3,
    enemy_index: i32,
) {
    let enemy = enemy_data.get(enemy_assets.get(enemy_index)).unwrap();
    commands
        .spawn_bundle(SpriteSheetBundle {
            transform: Transform {
                translation,
                scale: Vec3::splat(0.01),
                ..Default::default()
            },
            sprite: TextureAtlasSprite::new(0),
            texture_atlas: enemy.texture_atlas.clone(),
            ..Default::default()
        })
        .insert(Health::new(enemy.health))
        .insert(Enemy {
            damage: enemy.damage,
            speed: enemy.speed,
            attack_cooldown: enemy.attack_cooldown as u32,
            last_attack: 0,
        })
        .insert(AnimationTimer(Timer::from_seconds(0.1, true), 4))
        .insert(Rollback::new(rollback_id_provider.next_id()))
        .with_children(|parent| {
            parent.spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color: Color::DARK_GRAY,
                    custom_size: Some(Vec2::new(100., 5.1)),
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(0., 50., 1.)),
                ..default()
            });
            parent
                .spawn_bundle(SpriteBundle {
                    sprite: Sprite {
                        color: Color::RED,
                        custom_size: Some(Vec2::new(100., 5.1)),
                        ..default()
                    },
                    transform: Transform::from_translation(Vec3::new(0., 50., 2.)),
                    ..default()
                })
                .insert(HealthBar);
        });
}

fn fire_bullets(
    mut commands: Commands,
    inputs: Res<Vec<(u8, InputStatus)>>,
    images: Res<ImageAssets>,
    seed_frame: Res<SeedFrame>,
    mut player_query: Query<(Entity, &Transform, &Player, &mut Weapon, &MoveDir), Without<Dead>>,
    mut rip: ResMut<RollbackIdProvider>,
) {
    for (entity, transform, player, mut weapon, move_dir) in player_query.iter_mut() {
        let (input, _) = inputs[player.handle];
        if fire(input) && weapon.shoot(&seed_frame) {
            commands
                .spawn_bundle(SpriteBundle {
                    transform: Transform::from_translation(transform.translation.xy().extend(200.))
                        .with_rotation(Quat::from_rotation_arc_2d(Vec2::X, move_dir.0)),
                    texture: images.bullet.clone(),
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(0.3, 0.1)),
                        ..default()
                    },
                    ..default()
                })
                .insert(*move_dir)
                .insert(Bullet::new(100., entity))
                .insert(Rollback::new(rip.next_id()));
        }
    }
}

fn move_bullet(mut query: Query<(&mut Transform, &MoveDir), With<Bullet>>) {
    for (mut transform, dir) in query.iter_mut() {
        let delta = (dir.0 * 0.35).extend(0.);
        transform.translation += delta;
    }
}
