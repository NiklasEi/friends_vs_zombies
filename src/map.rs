use crate::loading::ImageAssets;
use crate::matchmaking::Seed;
use crate::MAP_SIZE;
use bevy::prelude::*;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, _app: &mut App) {
        // app.add_system_set(SystemSet::on_enter(GameState::InGame).with_system(setup));
    }
}

pub fn setup(mut commands: Commands, images: Res<ImageAssets>, seed: Res<Seed>) {
    info!("build map");
    let seed: [u8; 32] = [seed.0[1], seed.0[2]].repeat(16).try_into().unwrap();
    let mut rng = ChaCha8Rng::from_seed(seed);
    for row in 0..=MAP_SIZE {
        for column in 0..=MAP_SIZE {
            commands.spawn_bundle(SpriteSheetBundle {
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
                texture_atlas: images.grass.clone(),
                ..default()
            });
        }
    }
}
