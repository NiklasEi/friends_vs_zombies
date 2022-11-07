use crate::loading::FontAssets;
use crate::matchmaking::{LocalPlayer, PlayerReady, RemotePlayers};
use crate::menu::ButtonColors;
use crate::GameState;
use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(GameState::Matchmaking)
                .with_system(spawn_player_list)
                .with_system(spawn_ready_button),
        )
        .add_system_set(
            SystemSet::on_update(GameState::Matchmaking)
                .with_system(update_player_list)
                .with_system(click_ready_button),
        );
    }
}

#[derive(Component)]
struct PlayerList;

fn spawn_player_list(mut commands: Commands) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                position_type: PositionType::Relative,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },
            color: UiColor(Color::GOLD),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(TextBundle {
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
struct ReadyButton;

fn spawn_ready_button(
    mut commands: Commands,
    font_assets: Res<FontAssets>,
    button_colors: Res<ButtonColors>,
) {
    commands
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
        .insert(ReadyButton)
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                text: Text {
                    sections: vec![TextSection {
                        value: "Ready".to_string(),
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

fn click_ready_button(
    button_colors: Res<ButtonColors>,
    mut ready_state: ResMut<PlayerReady>,
    mut interaction_query: Query<
        (&Interaction, &mut UiColor),
        (Changed<Interaction>, With<ReadyButton>),
    >,
) {
    if ready_state.0 {
        return;
    }
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Clicked => {
                ready_state.0 = true;
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
                value: format!("{} {}\n", name, player.ready),
                style: TextStyle {
                    font: font_assets.fira_sans.clone(),
                    font_size: 20.0,
                    color: Color::rgb_u8(34, 32, 52),
                },
            })
        }
    }
}
