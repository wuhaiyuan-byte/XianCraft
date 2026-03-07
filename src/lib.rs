pub mod command;
pub mod npc;
pub mod world;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use tower_http::services::ServeDir;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::command::{parse, Command};
use crate::world::world::World;
use crate::world::player::Player;

#[derive(Deserialize)]
#[serde(tag = "type")]
enum ClientMessage {
    Login { user_id: String },
    Command { command: String },
}

#[derive(Serialize)]
#[serde(tag = "type")]
enum ServerMessage {
    Description { payload: String },
    Info { payload: String },
    Error { payload: String },
}

struct AppState {
    player_sessions: Mutex<HashMap<u64, PlayerSession>>,
    _game_state: Arc<GameState>,
    world: Arc<World>,
}

struct GameState {
    _counter: Mutex<i32>,
}

#[derive(Clone)]
struct PlayerSession {
    player: Player,
    user_id: Option<String>,
    current_room: String,
    #[allow(dead_code)]
    combat_state: String,
    sender: mpsc::Sender<Message>,
}

pub async fn run() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let game_state = Arc::new(GameState {
        _counter: Mutex::new(0),
    });

    let world = Arc::new(World::new());

    let app_state = Arc::new(AppState {
        player_sessions: Mutex::new(HashMap::new()),
        _game_state: game_state,
        world: world.clone(),
    });

    let game_loop_state = app_state.clone();
    tokio::spawn(async move {
        game_loop(game_loop_state).await;
    });

    let npc_loop_state = app_state.clone();
    tokio::spawn(async move {
        npc_loop(npc_loop_state).await;
    });

    let app = Router::new()
        .nest_service("/", ServeDir::new("client/dist"))
        .route("/ws", get(websocket_handler))
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("listening on {}", addr);
    axum::serve(tokio::net::TcpListener::bind(&addr).await.unwrap(), app.into_make_service())
        .await
        .unwrap();
}

async fn game_loop(app_state: Arc<AppState>) {
    loop {
        sleep(Duration::from_secs(10)).await;
        let player_count = app_state.player_sessions.lock().unwrap()
            .values()
            .filter(|s| s.user_id.is_some())
            .count();
        info!("{} players online.", player_count);
    }
}

async fn npc_loop(app_state: Arc<AppState>) {
    loop {
        sleep(Duration::from_secs(2)).await;
        let world_clone = app_state.world.clone();
        let mut npcs = world_clone.npcs.lock().unwrap().clone();
        for npc in npcs.values_mut() {
            npc.tick(world_clone.clone()).await;
        }
        *app_state.world.npcs.lock().unwrap() = npcs;
    }
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| websocket(socket, state))
}

fn get_full_room_description(room_id: &str, world: &Arc<World>) -> String {
    if let Some(room) = world.get_room(room_id) {
        let mut full_desc = room.get_description().to_string();

        // Add NPCs
        let npc_ids_in_room = room.get_npc_ids();
        if !npc_ids_in_room.is_empty() {
            let npcs_map = world.npcs.lock().unwrap();
            let mut npc_lines = Vec::new();
            for npc_id in npc_ids_in_room.iter() {
                if let Some(npc) = npcs_map.get(npc_id) {
                    npc_lines.push(npc.name.clone());
                }
            }
            if !npc_lines.is_empty() {
                full_desc.push('\n');
                full_desc.push_str(&npc_lines.join("\n"));
            }
        }

        // Add Exits
        let exits = room.get_exits();
        if !exits.is_empty() {
            let exit_keys: Vec<String> = exits.keys().cloned().collect();
            full_desc.push_str(&format!("\nExits: [{}]", exit_keys.join(", ")));
        }

        full_desc
    } else {
        "You are in a void.".to_string()
    }
}

async fn websocket(stream: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = stream.split();
    let (tx, mut rx) = mpsc::channel(100);

    let player_id = {
        let mut sessions = state.player_sessions.lock().unwrap();
        let mut new_id = 0;
        while sessions.contains_key(&new_id) {
            new_id += 1;
        }
        let player = Player::new(new_id, "".to_string()); // Name is set on login
        sessions.insert(new_id, PlayerSession {
            player,
            user_id: None,
            current_room: "town_1".to_string(),
            combat_state: "none".to_string(),
            sender: tx.clone(),
        });
        new_id
    };

    let starting_room = "town_1".to_string();
    state.world.get_room(&starting_room).unwrap().add_player(player_id);
    
    let desc = get_full_room_description(&starting_room, &state.world);
    let server_msg = ServerMessage::Description { payload: desc };
    if let Ok(msg_str) = serde_json::to_string(&server_msg) {
        let _ = tx.send(Message::Text(msg_str)).await;
    }

    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            Message::Text(t) => {
                match serde_json::from_str::<ClientMessage>(&t) {
                    Ok(client_msg) => match client_msg {
                        ClientMessage::Login { user_id } => {
                            let mut sessions = state.player_sessions.lock().unwrap();
                            if let Some(session) = sessions.get_mut(&player_id) {
                                session.user_id = Some(user_id.clone());
                                session.player.name = user_id;
                            }
                        }
                        ClientMessage::Command { command } => {
                             let (sender_clone, final_command) = {
                                let mut sessions = state.player_sessions.lock().unwrap();
                                let session = sessions.get_mut(&player_id).unwrap();
                                let mut parts = command.split_whitespace();
                                let first_word = parts.next().unwrap_or("");

                                let resolved_command = if let Some(resolved) = session.player.aliases.get(first_word) {
                                    let rest = parts.collect::<Vec<&str>>().join(" ");
                                    format!("{} {}", resolved, rest).trim().to_string()
                                } else {
                                    command.to_string()
                                };
                                (session.sender.clone(), resolved_command)
                            };
                            let cmd = parse(&final_command);
                            handle_command(cmd, player_id, state.clone(), sender_clone).await;
                        }
                    },
                    Err(_) => {
                        let sender_clone = {
                            let sessions = state.player_sessions.lock().unwrap();
                            sessions.get(&player_id).map(|s| s.sender.clone())
                        };
                        
                        if let Some(sender) = sender_clone {
                            let err_msg = ServerMessage::Error {
                                payload: "Invalid command format.".to_string(),
                            };
                            if let Ok(json_str) = serde_json::to_string(&err_msg) {
                                let _ = sender.send(Message::Text(json_str)).await;
                            }
                        }
                    }
                }
            }
            Message::Close(_) => {
                info!("Client {} disconnected", player_id);
                if let Some(session) = state.player_sessions.lock().unwrap().remove(&player_id) {
                    state.world.get_room(&session.current_room).unwrap().remove_player(player_id);
                }
                break;
            }
            _ => (),
        }
    }
}

async fn handle_command(command: Command, player_id: u64, state: Arc<AppState>, sender: mpsc::Sender<Message>) {
    let mut msg_to_send = None;

    // Block to limit the scope of the MutexGuard
    {
        let mut sessions = state.player_sessions.lock().unwrap();
        let session = match sessions.get_mut(&player_id) {
            Some(s) => s,
            None => return,
        };
        
        if session.user_id.is_none() {
            let err_msg = ServerMessage::Error { payload: "You must login first.".to_string() };
            msg_to_send = Some(err_msg);
        } else {
            let server_msg = match command {
                Command::Go { direction } => {
                    if let Some(next_id) = state.world.get_room(&session.current_room).and_then(|r| r.get_exits().get(&direction).cloned()) {
                        state.world.get_room(&session.current_room).unwrap().remove_player(player_id);
                        state.world.get_room(&next_id).unwrap().add_player(player_id);
                        session.current_room = next_id.to_string();

                        let new_room_desc = get_full_room_description(&next_id, &state.world);
                        Some(ServerMessage::Description { payload: new_room_desc })
                    } else {
                        Some(ServerMessage::Error { payload: "You can't go that way.".to_string() })
                    }
                }
                Command::Look => {
                    let room_desc = get_full_room_description(&session.current_room, &state.world);
                    Some(ServerMessage::Description { payload: room_desc })
                }
                Command::Attack { target } => {
                    let npc_id_to_attack = {
                        state.world.get_room(&session.current_room).and_then(|room| {
                            let npc_ids = room.get_npc_ids();
                            let npcs = state.world.npcs.lock().unwrap();
                            npc_ids.iter().find(|id| {
                                npcs.get(id).map_or(false, |npc| npc.name.to_lowercase() == target.to_lowercase())
                            }).cloned()
                        })
                    };

                    if let Some(npc_id) = npc_id_to_attack {
                        let mut npcs = state.world.npcs.lock().unwrap();
                        if let Some(npc) = npcs.get_mut(&npc_id) {
                            npc.combat_target = Some(player_id);
                            Some(ServerMessage::Info { payload: format!("You attack the {}.", npc.name) })
                        } else {
                            Some(ServerMessage::Error { payload: "You don't see that here.".to_string() })
                        }
                    } else {
                        Some(ServerMessage::Error { payload: "You don't see that here.".to_string() })
                    }
                }
                Command::Alias { name, command } => {
                    match (name, command) {
                        (Some(name), Some(command)) => {
                            session.player.aliases.insert(name.clone(), command.clone());
                            Some(ServerMessage::Info { payload: format!("Alias '{}' set to '{}'", name, command) })
                        }
                        (None, None) => {
                            if session.player.aliases.is_empty() {
                                Some(ServerMessage::Info { payload: "You have no aliases defined.".to_string() })
                            } else {
                                let alias_list = session.player.aliases.iter()
                                    .map(|(k, v)| format!("{}: {}", k, v))
                                    .collect::<Vec<String>>().join("\n");
                                Some(ServerMessage::Info { payload: format!("Your aliases:\n{}", alias_list) })
                            }
                        }
                        _ => Some(ServerMessage::Error { payload: "Invalid alias command.".to_string() })
                    }
                }
                Command::Unalias { name } => {
                    if session.player.aliases.remove(&name).is_some() {
                        Some(ServerMessage::Info { payload: format!("Alias '{}' removed.", name) })
                    } else {
                        Some(ServerMessage::Error { payload: format!("Alias '{}' not found.", name) })
                    }
                }
                Command::Invalid(reason) => Some(ServerMessage::Error { payload: reason }),
                _ => Some(ServerMessage::Error { payload: "Command not yet implemented.".to_string() }),
            };
            msg_to_send = server_msg;
        }
    }

    if let Some(msg) = msg_to_send {
        if let Ok(json_str) = serde_json::to_string(&msg) {
            let _ = sender.send(Message::Text(json_str)).await;
        }
    }
}
