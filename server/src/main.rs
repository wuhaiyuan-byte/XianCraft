use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::State,
    response::IntoResponse,
    routing::get,
    Router,
};
use futures_util::{
    sink::SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, atomic::{AtomicUsize, Ordering}},
    time::Duration,
};
use tokio::sync::{mpsc, RwLock};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// --- New, cleaner imports ---
mod world;
use world::{
    character::{Character, PlayerCharacter},
    room::{Room, BaseRoom},
};

//---------- Data Models & Messages ----------//

/// A message sent from the client to the server.
#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
enum ClientMessage {
    Login { username: String },
    Command { command: String },
}

/// Represents the data that gets saved for a player between sessions.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PersistentPlayerData {
    state: PlayerState,
    room_id: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlayerState {
    hp: u32,
    max_hp: u32,
    mp: u32,
    max_mp: u32,
    stamina: u32,
    max_stamina: u32,
}

#[derive(Serialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum ServerMessage {
    GameMessage { content: String },
    PlayerStateUpdate { state: PlayerState },
}


//---------- World State (Refactored) ----------//

#[derive(Debug)]
pub struct World {
    rooms: HashMap<usize, Box<dyn Room>>,
    characters: HashMap<usize, Box<dyn Character>>,
}

impl World {
    fn new() -> Self {
        let mut rooms: HashMap<usize, Box<dyn Room>> = HashMap::new();

        let great_hall = BaseRoom {
            id: 0,
            name: "Great Hall".to_string(),
            description: "You are in a vast, stone-walled great hall. A large fireplace crackles in the center.".to_string(),
            exits: [("north".to_string(), 1)].iter().cloned().collect(),
        };
        rooms.insert(great_hall.id, Box::new(great_hall));

        let narrow_corridor = BaseRoom {
            id: 1,
            name: "Narrow Corridor".to_string(),
            description: "A dimly lit corridor stretches before you. The air is damp and cool. The Great Hall is to the south.".to_string(),
            exits: [("south".to_string(), 0)].iter().cloned().collect(),
        };
        rooms.insert(narrow_corridor.id, Box::new(narrow_corridor));

        Self { 
            rooms, 
            characters: HashMap::new(),
        }
    }
}

//---------- Application State ----------//

#[derive(Clone)]
struct AppState {
    world: Arc<RwLock<World>>,
    next_character_id: Arc<AtomicUsize>,
    /// Our in-memory "database" for player data.
    /// Key: username (String), Value: PersistentPlayerData
    player_database: Arc<RwLock<HashMap<String, PersistentPlayerData>>>,
}

const SPAWN_ROOM_ID: usize = 0; // Great Hall

//---------- Main Application & Game Loop ----------//

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "server=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app_state = AppState {
        world: Arc::new(RwLock::new(World::new())),
        next_character_id: Arc::new(AtomicUsize::new(1)),
        // Initialize the in-memory database
        player_database: Arc::new(RwLock::new(HashMap::new())),
    };
    
    tokio::spawn(game_loop(app_state.clone()));

    let cors = CorsLayer::new().allow_origin(Any);
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .with_state(app_state)
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn game_loop(state: AppState) {
    let mut interval = tokio::time::interval(Duration::from_millis(500));
    loop {
        interval.tick().await;
        let mut world = state.world.write().await;

        for (_id, character) in world.characters.iter_mut() {
            let state = character.get_mut_state();
            if state.mp < state.max_mp {
                state.mp += 1;
            }
            
            let update_msg = ServerMessage::PlayerStateUpdate { state: character.get_state().clone() };
            let _ = character.get_sender().try_send(update_msg);
        }
    }
}

//---------- WebSocket Handling (Refactored for Login/Persistence) ----------//

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

/// This is the master function for a client connection.
/// It now handles the entire lifecycle: login, command processing, and saving on disconnect.
async fn handle_socket(socket: WebSocket, state: AppState) {
    let (ws_sender, ws_receiver) = socket.split();

    // This channel is for sending game updates TO the client's websocket.
    let (tx, rx) = mpsc::channel(100);
    tokio::spawn(forward_game_updates_to_ws(ws_sender, rx));

    // These will be determined after a successful login.
    let mut char_id: Option<usize> = None;
    let mut username: Option<String> = None;

    // This function now handles the login process and the main command loop.
    // It returns the character's ID and username if login was successful.
    if let Ok((id, u_name)) = handle_incoming_ws_messages(ws_receiver, tx, &state).await {
        char_id = Some(id);
        username = Some(u_name);
    }

    // --- Disconnection & Save Logic ---
    // This code runs when `handle_incoming_ws_messages` returns (i.e., the client disconnects).
    if let (Some(id), Some(uname)) = (char_id, username) {
        let mut world = state.world.write().await;
        // Get the character's final state from the world.
        if let Some(character) = world.characters.get(&id) {
            let data_to_persist = PersistentPlayerData {
                state: character.get_state().clone(),
                room_id: character.get_current_room_id(),
            };
            // Save the data to our in-memory database.
            let mut db = state.player_database.write().await;
            db.insert(uname.clone(), data_to_persist);
            tracing::debug!("Saved data for PlayerCharacter {} ({})", id, uname);
        }

        // Finally, remove the character from the active world.
        world.characters.remove(&id);
        tracing::debug!("PlayerCharacter {} disconnected from world", id);
    }
}

async fn forward_game_updates_to_ws(mut ws_sender: SplitSink<WebSocket, Message>, mut rx: mpsc::Receiver<ServerMessage>) {
    while let Some(msg) = rx.recv().await {
        let json_msg = serde_json::to_string(&msg).unwrap();
        if ws_sender.send(Message::Text(json_msg)).await.is_err() {
            break;
        }
    }
}

/// Handles the login process and the main command loop for a single player.
async fn handle_incoming_ws_messages(
    mut ws_receiver: SplitStream<WebSocket>,
    sender: mpsc::Sender<ServerMessage>,
    state: &AppState,
) -> Result<(usize, String), ()> { // Returns (char_id, username) on graceful disconnect

    // 1. --- Wait for Login Message ---
    if let Some(Ok(msg)) = ws_receiver.next().await {
        if let Message::Text(text) = msg {
            // The first message MUST be a login message.
            if let Ok(ClientMessage::Login { username }) = serde_json::from_str::<ClientMessage>(&text) {
                
                let char_id = state.next_character_id.fetch_add(1, Ordering::Relaxed);
                
                // 2. --- Load or Create Player Data ---
                let (player_char, is_new_player) = {
                    let db = state.player_database.read().await;
                    match db.get(&username) {
                        // Returning user: Load their saved data.
                        Some(data) => {
                            let player_char = PlayerCharacter {
                                id: char_id,
                                current_room_id: data.room_id,
                                state: data.state.clone(),
                                sender: sender.clone(),
                            };
                            (player_char, false)
                        }
                        // New user: Create a default character.
                        None => {
                             let player_char = PlayerCharacter {
                                id: char_id,
                                current_room_id: SPAWN_ROOM_ID,
                                state: PlayerState { hp: 100, max_hp: 100, mp: 45, max_mp: 50, stamina: 75, max_stamina: 100 },
                                sender: sender.clone(),
                            };
                            (player_char, true)
                        }
                    }
                };

                // 3. --- Add Character to the Live World ---
                {
                    let mut world = state.world.write().await;
                    world.characters.insert(char_id, Box::new(player_char));
                    tracing::debug!("PlayerCharacter {} ({}) added to world", char_id, &username);
                }

                // 4. --- Send Welcome Messages ---
                {
                    let world = state.world.read().await;
                    if let Some(character) = world.characters.get(&char_id) {
                         let current_room = world.rooms.get(&character.get_current_room_id()).unwrap();
                         let welcome_msg = if is_new_player {
                            format!("Welcome, new adventurer {}!\n{}\n\nType 'help' for commands.", username, current_room.get_description())
                         } else {
                            format!("Welcome back, {}!\n{}", username, current_room.get_description())
                         };

                        let _ = sender.send(ServerMessage::GameMessage { content: welcome_msg }).await;
                        let _ = sender.send(ServerMessage::PlayerStateUpdate { state: character.get_state().clone() }).await;
                    }
                }

                // 5. --- Main Command Loop ---
                while let Some(Ok(msg)) = ws_receiver.next().await {
                    if let Message::Text(text) = msg {
                         if let Ok(ClientMessage::Command { command }) = serde_json::from_str::<ClientMessage>(&text) {
                            let response_text = process_command(char_id, &command, &state).await;
                            let _ = sender.send(ServerMessage::GameMessage { content: response_text }).await;
                         } else {
                             let _ = sender.send(ServerMessage::GameMessage { content: "Invalid command format. Expecting {\"type\":\"Command\", ...}".to_string() }).await;
                         }
                    }
                }
                // Player disconnected, return their ID and username for the save process.
                return Ok((char_id, username));

            } else {
                 let _ = sender.send(ServerMessage::GameMessage { content: "Login failed. First message must be {\"type\":\"Login\", ...}".to_string() }).await;
            }
        }
    }
    // If login fails or connection closes abruptly before login.
    Err(())
}



async fn process_command(char_id: usize, command: &str, state: &AppState) -> String {
    let parts: Vec<&str> = command.trim().split_whitespace().collect();
    let command_word = parts.get(0);

    let (response, new_room_id) = {
        let world = state.world.read().await;
        let character = match world.characters.get(&char_id) {
            Some(c) => c,
            None => return "Error: Character not found.".to_string(),
        };

        match command_word {
            Some(&"look") => {
                let room = world.rooms.get(&character.get_current_room_id()).unwrap();
                let exits = room.get_exits().keys().map(|s| s.as_str()).collect::<Vec<_>>().join(", ");
                let response = format!("{} - {}\nExits: {}", room.get_name(), room.get_description(), exits);
                (response, None) 
            }
            Some(&"go") => {
                if let Some(direction) = parts.get(1) {
                    let room = world.rooms.get(&character.get_current_room_id()).unwrap();
                    if let Some(next_room_id) = room.get_exits().get(*direction) {
                        let next_room = world.rooms.get(next_room_id).unwrap();
                        let exits = next_room.get_exits().keys().map(|s| s.as_str()).collect::<Vec<_>>().join(", ");
                        let response = format!("{} - {}\nExits: {}", next_room.get_name(), next_room.get_description(), exits);
                        (response, Some(*next_room_id)) 
                    } else {
                        ("You can't go that way.".to_string(), None)
                    }
                } else {
                    ("Go where?".to_string(), None)
                }
            }
            Some(&"help") => {
                let help_text = "Available commands:\n  look         - See the description of your current room.\n  go [direction] - Move in a specific direction (e.g., go north).\n  help         - Show this help message.".to_string();
                (help_text, None)
            }
            _ => ("Unknown command. Type 'help' for a list of commands.".to_string(), None),
        }
    }; 

    if let Some(room_id) = new_room_id {
        let mut world = state.world.write().await;
        if let Some(character) = world.characters.get_mut(&char_id) {
            character.set_current_room_id(room_id);
        }
    } 

    response
}
