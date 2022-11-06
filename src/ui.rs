use crate::loading::FontAssets;
use crate::matchmaking::NewPlayerEvent;
use crate::GameState;
use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(GameState::Matchmaking).with_system(spawn_player_list),
        )
        .add_system_set(
            SystemSet::on_update(GameState::Matchmaking).with_system(update_player_list),
        );
    }
}

#[derive(Component)]
struct PlayerList;

fn spawn_player_list(mut commands: Commands) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Px(150.0), Val::Px(50.0)),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position: UiRect {
                    right: Val::Px(10.),
                    top: Val::Px(10.),
                    ..Default::default()
                },
                ..Default::default()
            },
            color: UiColor(Color::NONE),
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

fn update_player_list(
    mut list: Query<&mut Text, With<PlayerList>>,
    mut new_players: EventReader<NewPlayerEvent>,
    font_assets: Res<FontAssets>,
) {
    for new_player in new_players.iter() {
        list.single_mut().sections.push(TextSection {
            value: format!("{}\n", new_player.0.name),
            style: TextStyle {
                font: font_assets.fira_sans.clone(),
                font_size: 40.0,
                color: Color::rgb_u8(34, 32, 52),
            },
        })
    }
}
