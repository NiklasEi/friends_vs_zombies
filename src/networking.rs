use crate::loading::{EnemyAssets, PlayerAssets};
use crate::matchmaking::Seed;
use crate::players::AnimationTimer;
use crate::{
    direction, fire, game_input, Bullet, BulletReady, GameState, ImageAssets, MoveDir, Player,
    BULLET_RADIUS, MAP_SIZE, PLAYER_RADIUS,
};
use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy_ggrs::{GGRSPlugin, Rollback, RollbackIdProvider};
use ggrs::{InputStatus, P2PSession};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
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
                                .with_system(reload_bullet.after(advance_seed_frame))
                                .with_system(move_bullet.after(advance_seed_frame))
                                .with_system(fire_bullets.after(move_players).after(reload_bullet))
                                .with_system(kill_players.after(move_bullet).after(move_players)),
                        ),
                ),
            )
            .register_rollback_type::<Transform>()
            .register_rollback_type::<BulletReady>()
            .register_rollback_type::<MoveDir>()
            .register_rollback_type::<EnemyTimer>()
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
                texture_atlas: player_assets.player1.clone(),
                ..Default::default()
            })
            .insert(AnimationTimer(Timer::from_seconds(0.1, true), 4))
            .insert(Player { handle: player })
            .insert(BulletReady(true))
            .insert(MoveDir(-Vec2::X))
            .insert(Rollback::new(rollback_id_provider.next_id()));
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
    mut state: ResMut<State<GameState>>,
    player_query: Query<(Entity, &Transform), (With<Player>, Without<Bullet>)>,
    bullet_query: Query<&Transform, With<Bullet>>,
) {
    for (player, player_transform) in player_query.iter() {
        for bullet_transform in bullet_query.iter() {
            let distance = Vec2::distance(
                player_transform.translation.xy(),
                bullet_transform.translation.xy(),
            );
            if distance < PLAYER_RADIUS + BULLET_RADIUS {
                commands.entity(player).despawn_recursive();
                let _ = state.set(GameState::Interlude);
            }
        }
    }
}

fn move_players(
    inputs: Res<Vec<(u8, InputStatus)>>,
    mut player_query: Query<(&mut Transform, &mut MoveDir, &Player, &mut AnimationTimer)>,
) {
    for (mut transform, mut move_direction, player, mut animation_timer) in player_query.iter_mut()
    {
        let (input, _) = inputs[player.handle];
        let direction = direction(input);

        if direction == Vec2::ZERO {
            info!("pause {}", player.handle);
            animation_timer.0.pause();
            continue;
        }
        if animation_timer.0.paused() {
            info!("unpause {}", player.handle);
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
struct SeedFrame(u16);

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
    ]
    .repeat(8)
    .try_into()
    .unwrap();
    let mut rng = ChaCha8Rng::from_seed(seed);
    // if !rng.gen_bool(0.02) {
    //     return;
    // }
    let translation = Vec3::new(
        rng.gen_range(0..MAP_SIZE) as f32 - MAP_SIZE as f32 / 2.,
        rng.gen_range(0..MAP_SIZE) as f32 - MAP_SIZE as f32 / 2.,
        100.,
    );
    info!("Spawning enemy at {:?}", translation);
    commands
        .spawn_bundle(SpriteSheetBundle {
            transform: Transform {
                translation,
                scale: Vec3::splat(0.01),
                ..Default::default()
            },
            sprite: TextureAtlasSprite::new(0),
            texture_atlas: enemy_assets.enemy1.clone(),
            ..Default::default()
        })
        .insert(AnimationTimer(Timer::from_seconds(0.1, true), 4))
        .insert(Rollback::new(rollback_id_provider.next_id()));
}

fn reload_bullet(
    inputs: Res<Vec<(u8, InputStatus)>>,
    mut query: Query<(&mut BulletReady, &Player)>,
) {
    for (mut can_fire, player) in query.iter_mut() {
        let (input, _) = inputs[player.handle];
        if !fire(input) {
            can_fire.0 = true;
        }
    }
}

fn fire_bullets(
    mut commands: Commands,
    inputs: Res<Vec<(u8, InputStatus)>>,
    images: Res<ImageAssets>,
    mut player_query: Query<(&Transform, &Player, &mut BulletReady, &MoveDir)>,
    mut rip: ResMut<RollbackIdProvider>,
) {
    for (transform, player, mut bullet_ready, move_dir) in player_query.iter_mut() {
        let (input, _) = inputs[player.handle];
        if fire(input) && bullet_ready.0 {
            let player_pos = transform.translation.xy();
            let pos = player_pos + move_dir.0 * PLAYER_RADIUS + BULLET_RADIUS;
            commands
                .spawn_bundle(SpriteBundle {
                    transform: Transform::from_translation(pos.extend(200.))
                        .with_rotation(Quat::from_rotation_arc_2d(Vec2::X, move_dir.0)),
                    texture: images.bullet.clone(),
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(0.3, 0.1)),
                        ..default()
                    },
                    ..default()
                })
                .insert(*move_dir)
                .insert(Bullet)
                .insert(Rollback::new(rip.next_id()));
            bullet_ready.0 = false;
        }
    }
}

fn move_bullet(mut query: Query<(&mut Transform, &MoveDir), With<Bullet>>) {
    for (mut transform, dir) in query.iter_mut() {
        let delta = (dir.0 * 0.35).extend(0.);
        transform.translation += delta;
    }
}
