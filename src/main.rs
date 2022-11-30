// disable console on windows for release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(clippy::type_complexity)]

extern crate core;

use crate::enemies::EnemiesPlugin;
use crate::loading::{ImageAssets, LoadingPlugin};
use crate::map::MapPlugin;
use crate::matchmaking::MatchmakingPlugin;
use crate::menu::MenuPlugin;
use crate::networking::{GgrsConfig, InterludeTimer, NetworkingPlugin};
use crate::players::{LocalPlayerId, MoveDir, Player, PlayersPlugin, Weapon};
use crate::ui::UiPlugin;
use bevy::prelude::*;
use bevy::window::WindowId;
use bevy::winit::WinitWindows;
use input::*;
use std::io::Cursor;
use winit::window::Icon;

mod enemies;
mod input;
mod loading;
mod map;
mod matchmaking;
mod menu;
mod networking;
mod players;
mod ui;

const PLAYER_RADIUS: f32 = 0.5;
const REVIVE_DISTANCE: f32 = 1.2;
const ENEMY_RADIUS: f32 = 0.5;
const BULLET_RADIUS: f32 = 0.025;
const MAP_SIZE: i32 = 41;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    AssetLoading,
    Menu,
    Matchmaking,
    InGame,
    Interlude,
}

#[derive(Component, Reflect, Default)]
pub struct Bullet {
    damage: f64,
    max_hits: usize,
    already_hit: Vec<Entity>,
}

pub struct Score(pub u64);

impl Bullet {
    pub fn fire(damage: f64, shooter: Entity) -> Self {
        Bullet {
            damage,
            max_hits: 1,
            already_hit: vec![shooter],
        }
    }

    pub fn hit(&mut self, entity: Entity) -> bool {
        if self.already_hit.contains(&entity) {
            return false;
        }
        self.already_hit.push(entity);
        true
    }

    pub fn is_used_up(&self) -> bool {
        self.already_hit.len() > self.max_hits
    }
}

fn main() {
    let mut app = App::new();

    app.add_state(GameState::AssetLoading)
        .insert_resource(ClearColor(Color::rgb(0.53, 0.53, 0.53)))
        .insert_resource(WindowDescriptor {
            // fill the entire browser window
            fit_canvas_to_parent: true,
            canvas: Some("#bevy".to_owned()),
            ..default()
        })
        .add_startup_system(set_window_icon)
        .init_resource::<InterludeTimer>()
        .insert_resource(Score(0))
        .add_plugins(DefaultPlugins)
        .add_plugin(LoadingPlugin)
        .add_plugin(PlayersPlugin)
        .add_plugin(NetworkingPlugin)
        .add_plugin(MatchmakingPlugin)
        .add_plugin(MenuPlugin)
        .add_plugin(UiPlugin)
        .add_plugin(MapPlugin)
        .add_plugin(EnemiesPlugin)
        .run();
}

#[derive(PartialEq, Eq, Debug)]
enum GameMode {
    Single,
    Multi(bool),
}

// Sets the icon on windows and X11
fn set_window_icon(windows: NonSend<WinitWindows>) {
    let primary = windows.get_window(WindowId::primary()).unwrap();
    let icon_buf = Cursor::new(include_bytes!(
        "../build/macos/AppIcon.iconset/icon_256x256.png"
    ));
    if let Ok(image) = image::load(icon_buf, image::ImageFormat::Png) {
        let image = image.into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        let icon = Icon::from_rgba(rgba, width, height).unwrap();
        primary.set_window_icon(Some(icon));
    };
}
