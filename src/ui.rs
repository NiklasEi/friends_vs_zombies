use crate::loading::FontAssets;
use crate::matchmaking::{LocalPlayer, RemotePlayers, StartGame};
use crate::menu::{ButtonColors, GameCode};
use crate::networking::HealthBar;
use crate::players::Health;
use crate::{GameMode, GameState};
use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(GameState::Matchmaking)
                .with_system(spawn_player_list)
                .with_system(prepare_matchmaking_ui),
        )
        .add_system_set(
            SystemSet::on_update(GameState::Matchmaking)
                .with_system(update_player_list)
                .with_system(click_start_button),
        )
        .add_system_set(SystemSet::on_update(GameState::InGame).with_system(update_health_bars))
        .add_system_set(
            SystemSet::on_exit(GameState::Matchmaking).with_system(remove_matchmaking_only_ui),
        );
    }
}

#[derive(Component)]
struct PlayerList;

fn spawn_player_list(mut commands: Commands) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                size: Size {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                },
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            color: UiColor(Color::NONE),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(TextBundle {
                    style: Style {
                        margin: UiRect {
                            left: Val::Px(5.),
                            ..default()
                        },
                        ..default()
                    },
                    text: Text {
                        sections: vec![],
                        alignment: Default::default(),
                    },
                    ..Default::default()
                })
                .insert(PlayerList);
        });
}

#[derive(Component)]
struct MatchmakingOnly;

#[derive(Component)]
struct StartButton;

fn prepare_matchmaking_ui(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    game_mode: Res<GameMode>,
    game_code: Res<GameCode>,
    button_colors: Res<ButtonColors>,
) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                },
                flex_direction: FlexDirection::ColumnReverse,
                ..default()
            },
            color: UiColor(Color::NONE),
            ..default()
        })
        .with_children(|parent| {
            if *game_mode == GameMode::Multi(true) {
                parent
                    .spawn_bundle(ButtonBundle {
                        style: Style {
                            size: Size::new(Val::Px(120.0), Val::Px(50.0)),
                            margin: UiRect::all(Val::Auto),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..Default::default()
                        },
                        color: button_colors.normal,
                        ..Default::default()
                    })
                    .insert(StartButton)
                    .insert(MatchmakingOnly)
                    .with_children(|parent| {
                        parent.spawn_bundle(TextBundle {
                            text: Text {
                                sections: vec![TextSection {
                                    value: "Start".to_string(),
                                    style: TextStyle {
                                        font: font_assets.fira_sans.clone(),
                                        font_size: 40.0,
                                        color: Color::rgb(0.9, 0.9, 0.9),
                                    },
                                }],
                                alignment: TextAlignment::CENTER,
                            },
                            ..Default::default()
                        });
                    });
            } else if *game_mode == GameMode::Multi(false) {
                parent
                    .spawn_bundle(NodeBundle {
                        style: Style {
                            margin: UiRect::all(Val::Auto),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..Default::default()
                        },
                        color: UiColor(Color::NONE),
                        ..Default::default()
                    })
                    .insert(MatchmakingOnly)
                    .with_children(|parent| {
                        parent.spawn_bundle(TextBundle {
                            text: Text {
                                sections: vec![TextSection {
                                    value:
                                        "One player has a start button,\nwait for them to press it"
                                            .to_owned(),
                                    style: TextStyle {
                                        font: font_assets.fira_sans.clone(),
                                        font_size: 40.0,
                                        color: Color::rgb(0.9, 0.9, 0.9),
                                    },
                                }],
                                alignment: TextAlignment::CENTER,
                            },
                            ..Default::default()
                        });
                    });
            }

            parent
                .spawn_bundle(TextBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        position: UiRect {
                            top: Val::Px(15.),
                            left: Val::Px(15.),
                            ..default()
                        },
                        ..default()
                    },
                    text: Text {
                        sections: vec![TextSection {
                            value: format!("Game code: {}", game_code.0),
                            style: TextStyle {
                                font: font_assets.fira_sans.clone(),
                                font_size: 40.0,
                                color: Color::rgb(0.9, 0.9, 0.9),
                            },
                        }],
                        alignment: TextAlignment::CENTER,
                    },
                    ..Default::default()
                })
                .insert(MatchmakingOnly);
        });
}

fn remove_matchmaking_only_ui(mut commands: Commands, ui: Query<Entity, With<MatchmakingOnly>>) {
    for entity in &ui {
        commands.entity(entity).despawn_recursive();
    }
}

fn click_start_button(
    button_colors: Res<ButtonColors>,
    mut start_game: ResMut<StartGame>,
    mut interaction_query: Query<
        (&Interaction, &mut UiColor),
        (Changed<Interaction>, With<StartButton>),
    >,
) {
    if start_game.0 {
        return;
    }
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Clicked => {
                start_game.0 = true;
                *color = button_colors.selected;
            }
            Interaction::Hovered => {
                *color = button_colors.hovered;
            }
            Interaction::None => {
                *color = button_colors.normal;
            }
        }
    }
}

fn update_player_list(
    mut list: Query<&mut Text, With<PlayerList>>,
    players: Res<RemotePlayers>,
    local_player: Res<LocalPlayer>,
    font_assets: Res<FontAssets>,
) {
    if players.is_changed() {
        list.single_mut().sections.clear();
        for player in players.0.iter() {
            let name = if player.id == local_player.0.id {
                format!("{} (you)", player.name)
            } else {
                player.name.clone()
            };
            list.single_mut().sections.push(TextSection {
                value: format!("{}\n", name),
                style: TextStyle {
                    font: font_assets.fira_sans.clone(),
                    font_size: 20.0,
                    color: Color::rgb_u8(34, 32, 52),
                },
            })
        }
    }
}

pub fn update_health_bars(
    children: Query<(&Children, &Health), (Changed<Health>, Without<HealthBar>)>,
    mut bars: Query<&mut Transform, With<HealthBar>>,
) {
    for (children, health) in &children {
        for child in children.iter() {
            if let Ok(mut transform) = bars.get_mut(child.clone()) {
                transform.scale.x = (health.current / health.max) as f32;
            }
        }
    }
}
