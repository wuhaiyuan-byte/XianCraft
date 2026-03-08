pub mod command;
pub mod npc;
pub mod world;
pub mod world_model;

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
use crate::world::player::Player;
use crate::world::world_state::WorldState;

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

// AppState now holds the unified WorldState
struct AppState {
    world_state: WorldState,
    player_sessions: Mutex<HashMap<u64, PlayerSession>>,
}

#[derive(Clone)]
struct PlayerSession {
    player: Player,
    user_id: Option<String>,
    sender: mpsc::Sender<Message>,
}

pub async fn run(world_state: WorldState) {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app_state = Arc::new(AppState {
        world_state,
        player_sessions: Mutex::new(HashMap::new()),
    });

    let game_loop_state = app_state.clone();
    tokio::spawn(async move {
        game_loop(game_loop_state).await;
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
        let player_count = app_state.player_sessions.lock().unwrap().len();
        info!("{} players online.", player_count);
        // NPC logic can be added here later
    }
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| websocket(socket, state))
}

// Generates the full description for a room, including NPCs and exits.
fn get_full_room_description(room_id: &str, world_state: &WorldState) -> String {
    if let Some(room) = world_state.get_room(room_id) {
        let mut full_desc = room.description.clone();

        // Add NPCs in the room
        let npcs_in_room = world_state.get_npcs_in_room(room_id);
        if !npcs_in_room.is_empty() {
            let npc_names: Vec<String> = npcs_in_room.iter().map(|npc| npc.name.clone()).collect();
            full_desc.push_str("

👤 你看到了: ");
            full_desc.push_str(&npc_names.join(", "));
        }

        // Add Exits
        if !room.exits.is_empty() {
            let exit_keys: Vec<String> = room.exits.keys().cloned().collect();
            full_desc.push_str(&format!("

Exits: [{}]", exit_keys.join(", ")));
        }

        full_desc
    } else {
        "你身处一片虚无之中。".to_string()
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
        sessions.insert(
            new_id,
            PlayerSession {
                player: Player::new(new_id, "".to_string()),
                user_id: None,
                sender: tx.clone(),
            },
        );
        new_id
    };

    let starting_room = "genesis_altar".to_string();
    state.world_state.move_player_to_room(player_id, &starting_room);
    
    let welcome_message = format!("--- Welcome! Connecting to the realm... ---\n\n{}", get_full_room_description(&starting_room, &state.world_state));
    if let Ok(msg_str) = serde_json::to_string(&ServerMessage::Description { payload: welcome_message }) {
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
                            let sender_clone = {
                                let sessions = state.player_sessions.lock().unwrap();
                                sessions.get(&player_id).unwrap().sender.clone()
                            };
                            let cmd = parse(&command);
                            handle_command(cmd, player_id, state.clone(), sender_clone).await;
                        }
                    },
                    Err(_) => {
                        // Handle invalid message format
                    }
                }
            }
            Message::Close(_) => {
                info!("Client {} disconnected", player_id);
                state.player_sessions.lock().unwrap().remove(&player_id);
                state.world_state.dynamic_data.lock().unwrap().players.remove(&player_id);
                break;
            }
            _ => (),
        }
    }
}

async fn handle_command(command: Command, player_id: u64, state: Arc<AppState>, sender: mpsc::Sender<Message>) {
    let server_msg = {
        let session_lock = state.player_sessions.lock().unwrap();
        let session = match session_lock.get(&player_id) {
            Some(s) => s,
            None => return, // Player has disconnected, no-op
        };

        if session.user_id.is_none() {
            ServerMessage::Error { payload: "You must login first.".to_string() }
        } else {
            let world = &state.world_state;

            if let Some(current_room_id) = world.get_player_room_id(player_id) {
                if let Some(current_room) = world.get_room(&current_room_id) {
                    // All good, we have a valid room and room_id
                    match command {
                        Command::Go { direction } => {
                            if let Some(next_room_id) = current_room.exits.get(&direction) {
                                world.move_player_to_room(player_id, next_room_id);
                                let desc = get_full_room_description(next_room_id, world);
                                ServerMessage::Description { payload: desc }
                            } else {
                                ServerMessage::Error { payload: "You can't go that way.".to_string() }
                            }
                        }
                        Command::Look => {
                            let desc = get_full_room_description(&current_room_id, world);
                            ServerMessage::Description { payload: desc }
                        }
                        Command::Unknown(cmd) => ServerMessage::Error { payload: format!("Unknown command: {}", cmd) },
                        _ => ServerMessage::Info { payload: "Command not yet implemented.".to_string() },
                    }
                } else {
                    // Inconsistent state: player's room ID points to a non-existent room.
                    tracing::error!("Player {} is in an invalid room: {}", player_id, current_room_id);
                    ServerMessage::Error { payload: "Your location is corrupted. Please reconnect.".to_string() }
                }
            } else {
                // This should not happen if player is connected and logged in.
                tracing::error!("Could not find location for player {}", player_id);
                ServerMessage::Error { payload: "Critical error: Your location is unknown.".to_string() }
            }
        }
    };

    if let Ok(json_str) = serde_json::to_string(&server_msg) {
        let _ = sender.send(Message::Text(json_str)).await;
    }
}
