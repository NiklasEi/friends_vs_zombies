use crate::loading::ImageAssets;
use crate::matchmaking::Seed;
use crate::{GameState, MAP_SIZE};
use bevy::ecs::system::SystemState;
use bevy::prelude::*;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_exit(GameState::Matchmaking)
                .with_system(setup.exclusive_system().at_end()),
        );
    }
}

pub fn setup(world: &mut World) {
    let mut state: SystemState<(Res<ImageAssets>, Res<Seed>)> = SystemState::new(world);
    let (images, seed) = state.get(world);
    info!("build map");
    let seed: [u8; 32] = [seed.0[1], seed.0[2]].repeat(16).try_into().unwrap();
    let mut rng = ChaCha8Rng::from_seed(seed);
    let texture = images.grass.clone();
    for row in 0..=MAP_SIZE {
        for column in 0..=MAP_SIZE {
            world.spawn().insert_bundle(SpriteSheetBundle {
                transform: Transform {
                    translation: Vec3::new(
                        column as f32 - MAP_SIZE as f32 / 2.,
                        row as f32 - MAP_SIZE as f32 / 2.,
                        0.1,
                    ),
                    scale: Vec3::splat(0.1 / 3.1),
                    ..default()
                },
                sprite: TextureAtlasSprite {
                    index: rng.gen_range(0..32),
                    ..default()
                },
                texture_atlas: texture.clone(),
                ..default()
            });
        }
    }
}
