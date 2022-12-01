use crate::loading::{FontAssets, ImageAssets, PlayerAssets};
use crate::matchmaking::{LocalPlayer, RemotePlayers, StartGame};
use crate::menu::{ButtonColors, GameCode};
use crate::networking::{Dead, HealthBar};
use crate::players::{Health, Player};
use crate::{GameMode, GameState, Score};
use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;

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
        .add_system_set(SystemSet::on_exit(GameState::Matchmaking).with_system(prepare_game_ui))
        .add_system_set(
            SystemSet::on_update(GameState::InGame)
                .with_system(update_health_bars)
                .with_system(update_score)
                .with_system(move_player_markers),
        )
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

#[derive(Component)]
struct RootNode;

fn prepare_matchmaking_ui(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    image_assets: Res<ImageAssets>,
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
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            color: UiColor(Color::NONE),
            ..default()
        })
        .insert(RootNode)
        .with_children(|parent| {
            if *game_mode == GameMode::Multi(true) {
                parent.spawn_bundle(TextBundle {
                    text: Text {
                        sections: vec![TextSection {
                            value:
                            "Depending on your network,\nwait for up to one minute\nuntil all players are listed"
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
                }).insert(MatchmakingOnly);
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

            parent.spawn_bundle(ImageBundle {
                image: UiImage(image_assets.control.clone()),
                transform: Transform {
                    scale: Vec3::splat(0.5),
                    ..default()
                },
                ..default()
            }).insert(MatchmakingOnly);

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

#[derive(Component)]
struct ScoreText;

fn prepare_game_ui(mut commands: Commands, font_assets: Res<FontAssets>) {
    commands
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
                    value: "Score: 0".to_owned(),
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
        .insert(ScoreText);
}

fn update_score(score: Res<Score>, mut score_text: Query<&mut Text, With<ScoreText>>) {
    if !score.is_changed() {
        return;
    }
    if let Ok(mut text) = score_text.get_single_mut() {
        text.sections[0].value = format!("Score: {}", score.0);
    }
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

#[derive(Component, Reflect)]
pub struct PlayerMarker(pub Entity);

impl Default for PlayerMarker {
    fn default() -> Self {
        PlayerMarker(Entity::from_raw(0))
    }
}

fn move_player_markers(
    mut player_query: Query<(Entity, &Transform), (With<Player>, Without<PlayerMarker>)>,
    mut markers: Query<(
        &mut Transform,
        &PlayerMarker,
        &mut Visibility,
        &mut Handle<Image>,
    )>,
    dead: Query<&Dead>,
    windows: Res<Windows>,
    images: Res<PlayerAssets>,
    camera: Query<(&Camera, &Transform), (Without<Player>, Without<PlayerMarker>)>,
) {
    let (camera, center) = camera.single();
    let window = if let RenderTarget::Window(id) = camera.target {
        windows.get(id).unwrap()
    } else {
        windows.get_primary().unwrap()
    };

    let width = 10.;
    let height = 10. * (window.height() / window.width());
    for (mut marker_transform, marker, mut visibility, mut image) in markers.iter_mut() {
        let Ok((player_entity, player_transform)) = player_query.get_mut(marker.0) else {
            warn!("No player for marker O.o");
            continue;
        };
        if (center.translation.x - player_transform.translation.x).abs() < width
            && (center.translation.y - player_transform.translation.y).abs() < height
        {
            visibility.is_visible = false;
            continue;
        }
        if dead.contains(player_entity) {
            *image = images.marker_red.clone();
        } else {
            *image = images.marker.clone();
        }
        visibility.is_visible = true;
        let centered_position = (player_transform.translation.clone() - center.translation).xy();
        let angle = centered_position.angle_between(Vec2::X);
        marker_transform.rotation = Quat::from_rotation_z(-angle);
        let factor_x = (centered_position.x / width).abs();
        let factor_y = (centered_position.y / height).abs();
        let factor = if factor_x > factor_y {
            factor_x
        } else {
            factor_y
        };
        marker_transform.translation.x =
            center.translation.x + centered_position.x / (1.7 * factor);
        marker_transform.translation.y =
            center.translation.y + centered_position.y / (1.7 * factor);
    }
}

pub fn update_health_bars(
    player: Query<(Entity, &Health), (Changed<Health>, Without<HealthBar>)>,
    mut bars: Query<(&mut Transform, &HealthBar)>,
) {
    for (player, health) in &player {
        if let Some((mut transform, _)) = bars
            .iter_mut()
            .find(|(_, health_bar)| health_bar.0 == player)
        {
            transform.scale.x = (health.current / health.max) as f32;
            transform.translation.x = 50. * (health.current / health.max) as f32 - 50.;
        }
    }
}
