use crate::loading::{GameData, PlayerNames};
use crate::menu::GameCode;
use crate::{GameMode, GameState, GgrsConfig, InterludeTimer, LocalPlayerId};
use bevy::prelude::*;
use bevy::tasks::IoTaskPool;
use bevy_ggrs::SessionType;
use ggrs::PlayerType;
use matchbox_socket::WebRtcSocket;

pub struct MatchmakingPlugin;

impl Plugin for MatchmakingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RemotePlayers>()
            .init_resource::<StartGame>()
            .add_system_set(
                SystemSet::on_enter(GameState::Matchmaking).with_system(start_matchbox_socket),
            )
            .add_system_set(
                SystemSet::on_update(GameState::Matchmaking)
                    .with_system(wait_for_players)
                    .with_system(handle_packets.after(wait_for_players))
                    .with_system(build_ggrs_session.after(handle_packets)),
            );
    }
}

const START: u8 = 3;

fn start_matchbox_socket(
    mut commands: Commands,
    game_data: Res<GameData>,
    player_names: Res<Assets<PlayerNames>>,
    mut players: ResMut<RemotePlayers>,
    game_code: Res<GameCode>,
) {
    let room_url = format!("wss://match.nikl.me/{}", game_code.0);
    info!("connecting to matchbox server: {:?}", room_url);
    let (socket, message_loop) = WebRtcSocket::new(room_url);

    // The message loop needs to be awaited, or nothing will happen.
    // We do this here using bevy's task system.
    IoTaskPool::get().spawn(message_loop).detach();

    let player_names = player_names.get(&game_data.player_names).unwrap();
    let local_player = SocketPlayer {
        name: format!("{}", player_names.get_name_from_id(socket.id())),
        id: socket.id().clone(),
    };
    commands.insert_resource(LocalPlayer(local_player.clone()));
    players.0.push(local_player);

    commands.insert_resource(Some(socket));
}

#[derive(Default, Debug)]
pub struct RemotePlayers(pub Vec<SocketPlayer>);

pub struct LocalPlayer(pub SocketPlayer);

#[derive(Default)]
pub struct StartGame(pub bool);

#[derive(Debug, Clone)]
pub struct SocketPlayer {
    pub id: String,
    pub name: String,
}

fn handle_packets(
    mut socket: ResMut<Option<WebRtcSocket>>,
    mut start_game: ResMut<StartGame>,
    mut commands: Commands,
) {
    let Some(socket) = socket.as_mut() else {
        return;
    };
    let mut packets = socket.receive();
    packets
        .drain(..)
        .for_each(|(_, packet)| match packet.first().unwrap() {
            &START => {
                let seed = Seed([
                    *packet.get(1).unwrap_or(&0),
                    *packet.get(2).unwrap_or(&0),
                    *packet.get(3).unwrap_or(&0),
                ]);
                info!("let's go! {:?}", seed);
                commands.insert_resource(seed);
                start_game.0 = true;
            }
            _ => (),
        });
}

fn wait_for_players(
    mut socket: ResMut<Option<WebRtcSocket>>,
    mut players: ResMut<RemotePlayers>,
    game_data: Res<GameData>,
    player_names: Res<Assets<PlayerNames>>,
) {
    let Some(socket) = socket.as_mut() else {
        return;
    };

    // Check for new connections
    let mut new_players = socket.accept_new_connections();
    let socket_players = socket.players();
    let local_id = socket.id().clone();
    players.0.retain(|player| {
        player.id == local_id
            || socket_players
                .iter()
                .find(|socket_player| {
                    if let PlayerType::Remote(id) = socket_player {
                        id == &player.id
                    } else {
                        false
                    }
                })
                .is_some()
    });

    for player in new_players.drain(..) {
        info!("Player {} connected", player);
        let player_names = player_names.get(&game_data.player_names).unwrap();
        let new_player = SocketPlayer {
            name: player_names.get_name_from_id(&player),
            id: player.clone(),
        };
        players.0.push(new_player.clone());
    }
}

#[derive(Debug)]
pub struct Seed(pub [u8; 3]);

fn build_ggrs_session(
    mut commands: Commands,
    mut socket: ResMut<Option<WebRtcSocket>>,
    mut state: ResMut<State<GameState>>,
    mut interlude_timer: ResMut<InterludeTimer>,
    game_mode: Res<GameMode>,
    input: Res<Input<KeyCode>>,
    start_game: Res<StartGame>,
) {
    if socket.is_none() {
        return;
    }

    if *game_mode == GameMode::Multi(false) && !start_game.0 {
        return;
    }
    if *game_mode == GameMode::Multi(true) {
        if !input.pressed(KeyCode::Return) && !start_game.0 {
            return; // wait for more players
        } else {
            let seed = Seed([3, 4, 5]);
            let packet = Box::new([START, seed.0[0], seed.0[1], seed.0[2]]);
            commands.insert_resource(seed);
            let socket_players = socket.as_ref().as_ref().unwrap().players();
            for player in socket_players {
                if let PlayerType::Remote(id) = player {
                    socket.as_mut().as_mut().unwrap().send(packet.clone(), id);
                }
            }
        }
    }
    if *game_mode == GameMode::Single {
        let seed = Seed([3, 4, 5]);
        commands.insert_resource(seed);
    }
    let socket_players = socket.as_ref().as_ref().unwrap().players();
    let input_delay = if *game_mode == GameMode::Single { 0 } else { 2 };

    info!(
        "going in-game in {:?} mode with {} player(s)",
        *game_mode,
        socket_players.len()
    );

    // create a GGRS P2P session
    let mut session_builder = ggrs::SessionBuilder::<GgrsConfig>::new()
        .with_num_players(socket_players.len())
        .with_input_delay(input_delay);

    for (i, player) in socket_players.into_iter().enumerate() {
        if player == PlayerType::Local {
            commands.insert_resource(LocalPlayerId(i));
        }

        session_builder = session_builder
            .add_player(player.clone(), i)
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
