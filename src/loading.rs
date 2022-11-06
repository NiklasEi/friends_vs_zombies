use crate::GameState;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy_asset_loader::prelude::*;
use bevy_common_assets::json::JsonAssetPlugin;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(JsonAssetPlugin::<PlayerNames>::new(&["names"]))
            .add_loading_state(
                LoadingState::new(GameState::AssetLoading)
                    .with_collection::<ImageAssets>()
                    .with_collection::<FontAssets>()
                    .with_collection::<GameData>()
                    .continue_to_state(GameState::Menu),
            );
    }
}

#[derive(AssetCollection)]
pub struct GameData {
    #[asset(path = "player.names")]
    pub player_names: Handle<PlayerNames>,
}

#[derive(AssetCollection)]
pub struct ImageAssets {
    #[asset(path = "bullet.png")]
    pub bullet: Handle<Image>,
}

#[derive(AssetCollection)]
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
