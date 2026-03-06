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
use std::{
    collections::HashMap,
    sync::{Arc, atomic::{AtomicUsize, Ordering}},
};
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Data Models

#[derive(Clone, Debug)]
struct Room {
    id: usize,
    name: String,
    description: String,
    exits: HashMap<String, usize>, // Direction -> Room ID
}

#[derive(Clone, Debug)]
struct Player {
    id: usize,
    current_room_id: usize,
}

// World State

#[derive(Default, Debug)]
struct World {
    rooms: HashMap<usize, Room>,
    players: HashMap<usize, Player>,
}

impl World {
    fn new() -> Self {
        let mut rooms = HashMap::new();

        // 1. Initialize Rooms (Hardcoded)
        let great_hall = Room {
            id: 0,
            name: "Great Hall".to_string(),
            description: "You are in a vast, stone-walled great hall. A large fireplace crackles in the center.".to_string(),
            exits: [("north".to_string(), 1)].iter().cloned().collect(),
        };

        let narrow_corridor = Room {
            id: 1,
            name: "Narrow Corridor".to_string(),
            description: "A dimly lit corridor stretches before you. The air is damp and cool. The Great Hall is to the south.".to_string(),
            exits: [("south".to_string(), 0)].iter().cloned().collect(),
        };

        rooms.insert(great_hall.id, great_hall);
        rooms.insert(narrow_corridor.id, narrow_corridor);

        Self { 
            rooms, 
            players: HashMap::new(),
        }
    }
}

// Application State

#[derive(Clone)]
struct AppState {
    world: Arc<RwLock<World>>,
    next_player_id: Arc<AtomicUsize>,
}

const SPAWN_ROOM_ID: usize = 0; // Great Hall

#[tokio::main]
async fn main() {
    // Setup logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "server=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 2. Create Global Shared State
    let world = World::new();
    let app_state = AppState {
        world: Arc::new(RwLock::new(world)),
        next_player_id: Arc::new(AtomicUsize::new(1)),
    };
    
    // CORS Layer for allowing all origins (the fix for WebSocket errors)
    let cors = CorsLayer::new().allow_origin(Any);

    // Setup Axum Router
    let app = Router::new()
        .route("/ws", get(ws_handler)) // WebSocket Route
        .with_state(app_state)
        .layer(cors); // Apply the CORS layer

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let player_id = state.next_player_id.fetch_add(1, Ordering::Relaxed);
    let (mut sender, mut receiver): (SplitSink<WebSocket, Message>, SplitStream<WebSocket>) = socket.split();

    // Auto-assign to "spawn point"
    {
        let mut world = state.world.write().await;
        let player = Player {
            id: player_id,
            current_room_id: SPAWN_ROOM_ID,
        };
        world.players.insert(player_id, player);
        tracing::debug!("Player {} connected and added to world", player_id);
    }

    // Send initial room description with help hint
    let welcome_message = {
        let world = state.world.read().await;
        let spawn_room = world.rooms.get(&SPAWN_ROOM_ID).unwrap();
        format!("Welcome, Player {}!\n{}\n\nType 'help' for a list of commands.", player_id, spawn_room.description)
    };

    if sender.send(Message::Text(welcome_message)).await.is_err() {
        return; // client disconnected
    }
    
    // Main loop for this player
    while let Some(Ok(msg)) = receiver.next().await {
        if let Message::Text(text) = msg {
            let response = process_command(player_id, &text, &state).await;
            if sender.send(Message::Text(response)).await.is_err() {
                break; // client disconnected
            }
        }
    }

    // Player disconnected, remove them from the world
    state.world.write().await.players.remove(&player_id);
    tracing::debug!("Player {} disconnected", player_id);
}

async fn process_command(player_id: usize, command: &str, state: &AppState) -> String {
    let mut world = state.world.write().await;
    let mut player = match world.players.get_mut(&player_id) {
        Some(p) => p.clone(),
        None => return "Error: Player not found.".to_string(),
    };

    let parts: Vec<&str> = command.trim().split_whitespace().collect();
    let response = match parts.get(0) {
        Some(&"look") => {
            let room = world.rooms.get(&player.current_room_id).unwrap();
            let room_name = &room.name;
            let exits = room.exits.keys().map(|s| s.as_str()).collect::<Vec<_>>().join(", ");
            format!("{} - {}\nExits: {}", room_name, room.description, exits)
        }
        Some(&"go") => {
            if let Some(direction) = parts.get(1) {
                let room = world.rooms.get(&player.current_room_id).unwrap();
                if let Some(next_room_id) = room.exits.get(*direction) {
                    player.current_room_id = *next_room_id;
                    let next_room = world.rooms.get(next_room_id).unwrap();
                    let room_name = &next_room.name;
                    let exits = next_room.exits.keys().map(|s| s.as_str()).collect::<Vec<_>>().join(", ");
                    format!("{} - {}\nExits: {}", room_name, next_room.description, exits)
                } else {
                    "You can't go that way.".to_string()
                }
            } else {
                "Go where?".to_string()
            }
        }
        Some(&"help") => {
            "Available commands:\n  look         - See the description of your current room.\n  go [direction] - Move in a specific direction (e.g., go north).\n  help         - Show this help message.".to_string()
        }
        _ => "Unknown command. Type 'help' for a list of commands.".to_string(),
    };

    // Update player state
    world.players.insert(player_id, player);

    response
}
