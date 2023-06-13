use crate::GameState;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy_asset_loader::prelude::*;
use bevy_common_assets::json::JsonAssetPlugin;
use bevy_common_assets::ron::RonAssetPlugin;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::collections::HashMap;

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<EnemyData>()
            .add_plugin(JsonAssetPlugin::<PlayerNames>::new(&["names"]))
            .add_plugin(RonAssetPlugin::<CustomDynamicAssetCollection>::new(&[
                "my-assets",
            ]))
            .add_loading_state(
                LoadingState::new(GameState::AssetLoading).continue_to_state(GameState::Menu),
            )
            .add_collection_to_loading_state::<_, ImageAssets>(GameState::AssetLoading)
            .add_collection_to_loading_state::<_, FontAssets>(GameState::AssetLoading)
            .add_collection_to_loading_state::<_, GameData>(GameState::AssetLoading)
            .add_collection_to_loading_state::<_, PlayerAssets>(GameState::AssetLoading)
            .add_collection_to_loading_state::<_, EnemyAssets>(GameState::AssetLoading)
            .add_collection_to_loading_state::<_, AudioAssets>(GameState::AssetLoading)
            .add_dynamic_collection_to_loading_state::<_, CustomDynamicAssetCollection>(
                GameState::AssetLoading,
                "enemies.my-assets",
            );
    }
}

#[derive(AssetCollection, Resource)]
pub struct AudioAssets {
    #[asset(path = "audio/background.ogg")]
    pub background: Handle<AudioSource>,
    #[asset(path = "audio/enemy_fall.ogg")]
    pub enemy_fall: Handle<AudioSource>,
    #[asset(path = "audio/player_hit.ogg")]
    pub player_hit: Handle<AudioSource>,
    #[asset(path = "audio/player_hit_bullet.ogg")]
    pub player_hit_bullet: Handle<AudioSource>,
    #[asset(path = "audio/lost.ogg")]
    pub lost: Handle<AudioSource>,
    #[asset(path = "audio/pew.ogg")]
    pub pew: Handle<AudioSource>,
    #[asset(path = "audio/revive.ogg")]
    pub revive: Handle<AudioSource>,
}

#[derive(AssetCollection, Resource)]
pub struct GameData {
    #[asset(path = "player.names")]
    pub player_names: Handle<PlayerNames>,
}

#[derive(AssetCollection, Resource)]
pub struct ImageAssets {
    #[asset(path = "bullet.png")]
    pub bullet: Handle<Image>,
    #[asset(path = "control.png")]
    pub control: Handle<Image>,
    #[asset(texture_atlas(tile_size_x = 32., tile_size_y = 32., columns = 8, rows = 8))]
    #[asset(path = "grass.png")]
    pub grass: Handle<TextureAtlas>,
}

#[derive(AssetCollection, Resource)]
pub struct PlayerAssets {
    #[asset(path = "players/marker.png")]
    pub marker: Handle<Image>,
    #[asset(path = "players/marker_red.png")]
    pub marker_red: Handle<Image>,
    #[asset(texture_atlas(tile_size_x = 96., tile_size_y = 96., columns = 4, rows = 1))]
    #[asset(path = "players/player1.png")]
    pub player1: Handle<TextureAtlas>,
    #[asset(texture_atlas(tile_size_x = 96., tile_size_y = 96., columns = 4, rows = 1))]
    #[asset(path = "players/player2.png")]
    pub player2: Handle<TextureAtlas>,
    #[asset(texture_atlas(tile_size_x = 96., tile_size_y = 96., columns = 4, rows = 1))]
    #[asset(path = "players/player3.png")]
    pub player3: Handle<TextureAtlas>,
    #[asset(texture_atlas(tile_size_x = 96., tile_size_y = 96., columns = 4, rows = 1))]
    #[asset(path = "players/player4.png")]
    pub player4: Handle<TextureAtlas>,
    #[asset(texture_atlas(tile_size_x = 96., tile_size_y = 96., columns = 4, rows = 1))]
    #[asset(path = "players/player5.png")]
    pub player5: Handle<TextureAtlas>,
    #[asset(texture_atlas(tile_size_x = 96., tile_size_y = 96., columns = 4, rows = 1))]
    #[asset(path = "players/player6.png")]
    pub player6: Handle<TextureAtlas>,
    #[asset(texture_atlas(tile_size_x = 96., tile_size_y = 96., columns = 4, rows = 1))]
    #[asset(path = "players/player7.png")]
    pub player7: Handle<TextureAtlas>,
    #[asset(texture_atlas(tile_size_x = 96., tile_size_y = 96., columns = 4, rows = 1))]
    #[asset(path = "players/player8.png")]
    pub player8: Handle<TextureAtlas>,
    #[asset(texture_atlas(tile_size_x = 96., tile_size_y = 96., columns = 4, rows = 1))]
    #[asset(path = "players/player9.png")]
    pub player9: Handle<TextureAtlas>,
    #[asset(texture_atlas(tile_size_x = 96., tile_size_y = 96., columns = 4, rows = 1))]
    #[asset(path = "players/player10.png")]
    pub player10: Handle<TextureAtlas>,
}

impl PlayerAssets {
    pub fn get_atlas(&self, player: usize) -> &Handle<TextureAtlas> {
        match player % 10 {
            0 => &self.player1,
            1 => &self.player2,
            2 => &self.player3,
            3 => &self.player4,
            4 => &self.player5,
            5 => &self.player6,
            6 => &self.player7,
            7 => &self.player8,
            8 => &self.player9,
            9 => &self.player10,
            _ => panic!("Whuuut?"),
        }
    }
}

#[derive(AssetCollection, Resource)]
pub struct EnemyAssets {
    #[asset(key = "devil")]
    pub devil: Handle<EnemyData>,
    #[asset(key = "zombie")]
    pub zombie: Handle<EnemyData>,
}

impl EnemyAssets {
    pub fn get(&self, random_index: i32) -> &Handle<EnemyData> {
        if random_index > 80 {
            &self.devil
        } else {
            &self.zombie
        }
    }
}

#[derive(AssetCollection, Resource)]
pub struct FontAssets {
    #[asset(path = "fonts/FiraSans-Bold.ttf")]
    pub fira_sans: Handle<Font>,
}

#[derive(serde::Deserialize, TypeUuid)]
#[uuid = "7fd2fcba-af36-c126-4692-b29b2d9b78b9"]
pub struct PlayerNames(Vec<String>);

impl PlayerNames {
    pub fn get_name_from_id(&self, id: &str) -> String {
        let (a, b) = id.as_bytes().split_at(32);
        // SAFETY: a points to [T; N]? Yes it's [T] of length N (checked by split_at)
        let (seed, _) = unsafe { (&*(a.as_ptr() as *const [u8; 32]), b) };
        let mut rng = ChaCha8Rng::from_seed(seed.clone());
        let index: usize = rng.gen_range(0..self.0.len());
        let name = self.0.get(index).unwrap().clone();
        info!("Chose name {} for id {}", name, id);
        name
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
enum CustomDynamicAsset {
    Enemy {
        sprite_sheet: String,
        speed: f32,
        damage: f64,
        health: f64,
        attack_cooldown: u8,
    },
}

#[derive(TypeUuid)]
#[uuid = "7fd2fcba-df98-c126-4692-b29b2d9b78b9"]
pub struct EnemyData {
    pub texture_atlas: Handle<TextureAtlas>,
    pub speed: f32,
    pub damage: f64,
    pub health: f64,
    pub attack_cooldown: u8,
}

impl DynamicAsset for CustomDynamicAsset {
    fn load(&self, asset_server: &AssetServer) -> Vec<HandleUntyped> {
        match self {
            CustomDynamicAsset::Enemy { sprite_sheet, .. } => {
                vec![asset_server.load_untyped(sprite_sheet)]
            }
        }
    }

    fn build(&self, world: &mut World) -> Result<DynamicAssetType, anyhow::Error> {
        let cell = world.cell();
        let asset_server = cell
            .get_resource::<AssetServer>()
            .expect("Failed to get asset server");
        match self {
            CustomDynamicAsset::Enemy {
                sprite_sheet,
                speed,
                damage,
                health,
                attack_cooldown,
            } => {
                let mut atlases = cell
                    .get_resource_mut::<Assets<TextureAtlas>>()
                    .expect("Failed to get TextureAtlas assets");
                let mut enemies = cell
                    .get_resource_mut::<Assets<EnemyData>>()
                    .expect("Failed to get EnemyData assets");
                let atlas = TextureAtlas::from_grid(
                    asset_server.load(sprite_sheet),
                    Vec2::splat(96.),
                    4,
                    1,
                    None,
                    None,
                );

                Ok(DynamicAssetType::Single(
                    enemies
                        .add(EnemyData {
                            texture_atlas: atlases.add(atlas),
                            speed: *speed,
                            attack_cooldown: *attack_cooldown,
                            damage: *damage,
                            health: *health,
                        })
                        .clone_untyped(),
                ))
            }
        }
    }
}

#[derive(serde::Deserialize, bevy::reflect::TypeUuid)]
#[uuid = "18dc82eb-d5f5-4d72-b0c4-e2b234367c35"]
pub struct CustomDynamicAssetCollection(HashMap<String, CustomDynamicAsset>);

impl DynamicAssetCollection for CustomDynamicAssetCollection {
    fn register(&self, dynamic_assets: &mut DynamicAssets) {
        for (key, asset) in self.0.iter() {
            dynamic_assets.register_asset(key, Box::new(asset.clone()));
        }
    }
}
