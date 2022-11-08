use crate::loading::FontAssets;
use crate::{GameMode, GameState};
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use rand::{thread_rng, Rng};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ButtonColors>()
            .insert_resource(GameCode("".to_owned()))
            .add_system_set(SystemSet::on_enter(GameState::Menu).with_system(setup_menu))
            .add_system_set(
                SystemSet::on_update(GameState::Menu)
                    .with_system(click_singleplayer_button)
                    .with_system(click_create_game_button)
                    .with_system(click_join_game_button)
                    .with_system(listen_for_game_code),
            )
            .add_system_set(SystemSet::on_exit(GameState::Menu).with_system(cleanup_menu));
    }
}

pub struct ButtonColors {
    pub normal: UiColor,
    pub hovered: UiColor,
    pub selected: UiColor,
}

impl Default for ButtonColors {
    fn default() -> Self {
        ButtonColors {
            normal: Color::rgb(0.15, 0.15, 0.15).into(),
            hovered: Color::rgb(0.25, 0.25, 0.25).into(),
            selected: Color::rgb(0.55, 0.55, 0.55).into(),
        }
    }
}

#[derive(Component)]
struct SingleplayerButton;

#[derive(Component)]
struct CreateGameButton;

#[derive(Component)]
struct JoinGameButton;

#[derive(Component)]
struct MenuUi;

#[derive(Component)]
struct CodeDisplay;

pub struct GameCode(pub(crate) String);

fn setup_menu(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    button_colors: Res<ButtonColors>,
) {
    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.projection.scaling_mode = ScalingMode::FixedVertical(10.);
    commands.spawn_bundle(camera_bundle);
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
        .insert(MenuUi)
        .with_children(|parent| {
            parent
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(250.0), Val::Px(50.0)),
                        margin: UiRect {
                            bottom: Val::Px(15.),
                            ..UiRect::all(Val::Auto)
                        },
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    color: button_colors.normal,
                    ..Default::default()
                })
                .insert(SingleplayerButton)
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text {
                            sections: vec![TextSection {
                                value: "Play alone".to_string(),
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

            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Px(250.0), Val::Px(50.0)),
                        margin: UiRect::all(Val::Auto),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    color: UiColor(Color::NONE),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text {
                            sections: vec![TextSection {
                                value: "Multiplayer".to_string(),
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
            parent
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(250.0), Val::Px(50.0)),
                        margin: UiRect::all(Val::Auto),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    color: button_colors.normal,
                    ..Default::default()
                })
                .insert(CreateGameButton)
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text {
                            sections: vec![TextSection {
                                value: "New Game".to_string(),
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
            parent
                .spawn_bundle(TextBundle {
                    text: Text {
                        sections: vec![TextSection {
                            value: "Code: ______".to_owned(),
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
                .insert(CodeDisplay);
            parent
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Px(250.0), Val::Px(50.0)),
                        margin: UiRect::all(Val::Auto),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    color: button_colors.selected,
                    ..Default::default()
                })
                .insert(JoinGameButton)
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text {
                            sections: vec![TextSection {
                                value: "Join Game".to_string(),
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
        });
}

fn click_singleplayer_button(
    mut commands: Commands,
    button_colors: Res<ButtonColors>,
    mut state: ResMut<State<GameState>>,
    mut interaction_query: Query<
        (&Interaction, &mut UiColor),
        (Changed<Interaction>, With<SingleplayerButton>),
    >,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Clicked => {
                state.set(GameState::Matchmaking).unwrap();
                commands.insert_resource(GameMode::Single);
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

fn click_create_game_button(
    mut commands: Commands,
    button_colors: Res<ButtonColors>,
    mut state: ResMut<State<GameState>>,
    mut interaction_query: Query<
        (&Interaction, &mut UiColor),
        (Changed<Interaction>, With<CreateGameButton>),
    >,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Clicked => {
                state.set(GameState::Matchmaking).unwrap();
                commands.insert_resource(GameMode::Multi(true));
                commands.insert_resource(GameCode(build_game_code()));
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

fn build_game_code() -> String {
    let mut code = "".to_owned();
    let mut random = thread_rng();
    for _ in 0..6 {
        code = format!(
            "{}{}",
            code,
            KEY_CODES[random.gen_range(0..KEY_CODES.len())].1
        );
    }

    code
}

const KEY_CODES: [KeyCodeOptions; 21] = [
    (KeyCode::A, "A"),
    (KeyCode::B, "B"),
    (KeyCode::C, "C"),
    (KeyCode::D, "D"),
    (KeyCode::E, "E"),
    (KeyCode::F, "F"),
    (KeyCode::G, "G"),
    (KeyCode::H, "H"),
    (KeyCode::K, "K"),
    (KeyCode::M, "M"),
    (KeyCode::N, "N"),
    (KeyCode::O, "O"),
    (KeyCode::P, "P"),
    (KeyCode::Q, "Q"),
    (KeyCode::R, "R"),
    (KeyCode::S, "S"),
    (KeyCode::T, "T"),
    (KeyCode::W, "W"),
    (KeyCode::X, "X"),
    (KeyCode::Y, "Y"),
    (KeyCode::Z, "Z"),
];
type KeyCodeOptions = (KeyCode, &'static str);

fn listen_for_game_code(
    mut code: Query<&mut Text, With<CodeDisplay>>,
    input: Res<Input<KeyCode>>,
    mut game_code: ResMut<GameCode>,
) {
    if input.just_pressed(KeyCode::Back) {
        game_code.0.pop();
    }
    input.get_just_pressed().for_each(|key_code| {
        if let Some((_, value)) = KEY_CODES.iter().find(|(code, _)| code == key_code) {
            if game_code.0.len() < 6 {
                game_code.0 = format!("{}{}", game_code.0, value);
            }
        }
    });
    code.single_mut().sections[0].value =
        format!("Code: {}{}", game_code.0, "_".repeat(6 - game_code.0.len()))
}

fn click_join_game_button(
    mut commands: Commands,
    button_colors: Res<ButtonColors>,
    game_code: Res<GameCode>,
    mut state: ResMut<State<GameState>>,
    mut interaction_query: Query<
        (&Interaction, &mut UiColor),
        (Changed<Interaction>, With<JoinGameButton>),
    >,
) {
    if game_code.0.len() < 6 {
        return;
    }
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Clicked => {
                state.set(GameState::Matchmaking).unwrap();
                commands.insert_resource(GameMode::Multi(false));
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

fn cleanup_menu(mut commands: Commands, button: Query<Entity, With<MenuUi>>) {
    for entity in button.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
