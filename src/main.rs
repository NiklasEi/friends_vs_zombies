#![allow(clippy::type_complexity)]

use crate::loading::{ImageAssets, LoadingPlugin};
use crate::matchmaking::MatchmakingPlugin;
use crate::menu::MenuPlugin;
use crate::networking::{GgrsConfig, InterludeTimer, NetworkingPlugin};
use crate::players::{BulletReady, LocalPlayerId, MoveDir, Player, PlayersPlugin};
use bevy::prelude::*;
use input::*;

mod input;
mod loading;
mod matchmaking;
mod menu;
mod networking;
mod players;

const PLAYER_RADIUS: f32 = 0.5;
const BULLET_RADIUS: f32 = 0.025;
const MAP_SIZE: i32 = 41;
const GRID_WIDTH: f32 = 0.05;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    AssetLoading,
    Menu,
    Matchmaking,
    InGame,
    Interlude,
}

#[derive(Component, Reflect, Default)]
pub struct Bullet;

fn main() {
    let mut app = App::new();

    app.add_state(GameState::AssetLoading)
        .insert_resource(ClearColor(Color::rgb(0.53, 0.53, 0.53)))
        .insert_resource(WindowDescriptor {
            // fill the entire browser window
            fit_canvas_to_parent: true,
            ..default()
        })
        .init_resource::<InterludeTimer>()
        .add_plugins(DefaultPlugins)
        .add_plugin(LoadingPlugin)
        .add_plugin(PlayersPlugin)
        .add_plugin(NetworkingPlugin)
        .add_plugin(MatchmakingPlugin)
        .add_plugin(MenuPlugin)
        .run();
}

#[derive(PartialEq, Eq, Debug)]
enum GameMode {
    Single,
    Multi,
}
