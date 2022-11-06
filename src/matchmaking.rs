use crate::loading::{GameData, PlayerNames};
use crate::{GameMode, GameState, GgrsConfig, InterludeTimer, LocalPlayerId, GRID_WIDTH, MAP_SIZE};
use bevy::prelude::*;
use bevy::tasks::IoTaskPool;
use bevy_ggrs::SessionType;
use ggrs::PlayerType;
use matchbox_socket::WebRtcSocket;

pub struct MatchmakingPlugin;

impl Plugin for MatchmakingPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<NewPlayerEvent>()
            .init_resource::<RemotePlayers>()
            .add_system_set(
                SystemSet::on_enter(GameState::Matchmaking)
                    .with_system(start_matchbox_socket)
                    .with_system(setup),
            )
            .add_system_set(
                SystemSet::on_update(GameState::Matchmaking).with_system(wait_for_players),
            );
    }
}

fn setup(mut commands: Commands) {
    // Horizontal lines
    for i in 0..=MAP_SIZE {
        commands.spawn_bundle(SpriteBundle {
            transform: Transform::from_translation(Vec3::new(
                0.,
                i as f32 - MAP_SIZE as f32 / 2.,
                0.,
            )),
            sprite: Sprite {
                color: Color::rgb(0.27, 0.27, 0.27),
                custom_size: Some(Vec2::new(MAP_SIZE as f32, GRID_WIDTH)),
                ..default()
            },
            ..default()
        });
    }

    // Vertical lines
    for i in 0..=MAP_SIZE {
        commands.spawn_bundle(SpriteBundle {
            transform: Transform::from_translation(Vec3::new(
                i as f32 - MAP_SIZE as f32 / 2.,
                0.,
                0.,
            )),
            sprite: Sprite {
                color: Color::rgb(0.27, 0.27, 0.27),
                custom_size: Some(Vec2::new(GRID_WIDTH, MAP_SIZE as f32)),
                ..default()
            },
            ..default()
        });
    }
}

fn start_matchbox_socket(
    mut commands: Commands,
    game_data: Res<GameData>,
    player_names: Res<Assets<PlayerNames>>,
    mut new_player_events: EventWriter<NewPlayerEvent>,
) {
    let room_url = "wss://match.nikl.me/bevy_boxhead";
    info!("connecting to matchbox server: {:?}", room_url);
    let (socket, message_loop) = WebRtcSocket::new(room_url);

    // The message loop needs to be awaited, or nothing will happen.
    // We do this here using bevy's task system.
    IoTaskPool::get().spawn(message_loop).detach();

    let player_names = player_names.get(&game_data.player_names).unwrap();
    let new_player = RemotePlayer {
        name: format!("{} (you)", player_names.get_name_from_id(socket.id())),
        id: socket.id().clone(),
    };
    new_player_events.send(NewPlayerEvent(new_player));

    commands.insert_resource(Some(socket));
}

#[derive(Default, Debug)]
pub struct RemotePlayers(Vec<RemotePlayer>);

pub struct NewPlayerEvent(pub RemotePlayer);

#[derive(Debug, Clone)]
pub struct RemotePlayer {
    pub id: String,
    pub name: String,
}

fn wait_for_players(
    mut commands: Commands,
    mut socket: ResMut<Option<WebRtcSocket>>,
    mut state: ResMut<State<GameState>>,
    mut interlude_timer: ResMut<InterludeTimer>,
    game_mode: Res<GameMode>,
    input: Res<Input<KeyCode>>,
    mut players: ResMut<RemotePlayers>,
    game_data: Res<GameData>,
    player_names: Res<Assets<PlayerNames>>,
    mut new_player_events: EventWriter<NewPlayerEvent>,
) {
    let socket = socket.as_mut();

    // If there is no socket we've already started the game
    if socket.is_none() {
        return;
    }

    // Check for new connections
    let mut new_players = socket.as_mut().unwrap().accept_new_connections();
    let socket_players = socket.as_ref().unwrap().players();

    for player in new_players.drain(..) {
        info!("Player {} connected", player);
        let player_names = player_names.get(&game_data.player_names).unwrap();
        let new_player = RemotePlayer {
            name: player_names.get_name_from_id(&player),
            id: player,
        };
        players.0.push(new_player.clone());
        new_player_events.send(NewPlayerEvent(new_player));
    }

    if *game_mode == GameMode::Multi && !input.pressed(KeyCode::NumpadEnter) {
        return; // wait for more players
    }
    let num_players = if *game_mode == GameMode::Multi {
        socket_players.len()
    } else {
        1
    };
    let input_delay = if *game_mode == GameMode::Multi { 2 } else { 0 };

    info!(
        "going in-game in {:?} mode with {} player(s)",
        *game_mode, num_players
    );

    // create a GGRS P2P session
    let mut session_builder = ggrs::SessionBuilder::<GgrsConfig>::new()
        .with_num_players(num_players)
        .with_input_delay(input_delay);

    for (i, player) in socket_players.into_iter().enumerate() {
        if player == PlayerType::Local {
            commands.insert_resource(LocalPlayerId(i));
        }

        session_builder = session_builder
            .add_player(player, i)
            .expect("failed to add player");
    }

    // move the socket out of the resource (required because GGRS takes ownership of it)
    let socket = socket.take().unwrap();

    // start the GGRS session
    let session = session_builder
        .start_p2p_session(socket)
        .expect("failed to start session");

    commands.insert_resource(session);
    commands.insert_resource(SessionType::P2PSession);

    interlude_timer.0 = 3;
    state.set(GameState::Interlude).unwrap();
}
