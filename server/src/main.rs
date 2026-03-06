use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::State,
    response::IntoResponse,
    routing::get,
    Router,
};
use futures_util::{
    sink::SinkExt,
    stream::{SplitSink, StreamExt},
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

// --- World-related imports ---
mod world;
use world::{
    combat::{self, AttackResult, Combatant}, // <--- CORRECTED: Imported the Combatant trait
    entity::Entity,
    npc::Npc,
    player::Player,
    player_state::{DerivedStats, PlayerState},
    room::{Room, BaseRoom},
};

//---------- Data Models & Messages ----------//

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
enum ClientMessage {
    Login { username: String },
    Command { command: String },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PersistentPlayerData {
    state: PlayerState,
    room_id: usize,
}

#[derive(Serialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum ServerMessage {
    GameMessage { content: String },
    FullState { state: PlayerState },
    DerivedStatsUpdate { state: DerivedStats },
    CombatUpdate(AttackResult), // This can be used for UI updates (e.g., health bars)
}

//---------- World State & Managers ----------//

#[derive(Debug)]
pub struct World {
    rooms: HashMap<usize, Box<dyn Room>>,
    pub entities: HashMap<usize, Box<dyn Entity>>,
    active_combats: Vec<(usize, usize)>,
}

impl World {
    fn new() -> Self {
        let mut rooms: HashMap<usize, Box<dyn Room>> = HashMap::new();

        let great_hall = BaseRoom {
            id: 0,
            name: "Great Hall".to_string(),
            description: "A vast, stone-walled great hall with a large fireplace.".to_string(),
            exits: [("north".to_string(), 1)].iter().cloned().collect(),
        };
        rooms.insert(great_hall.id, Box::new(great_hall));

        let narrow_corridor = BaseRoom {
            id: 1,
            name: "Narrow Corridor".to_string(),
            description: "A dimly lit corridor. The Great Hall is to the south.".to_string(),
            exits: [("south".to_string(), 0)].iter().cloned().collect(),
        };
        rooms.insert(narrow_corridor.id, Box::new(narrow_corridor));

        Self { 
            rooms, 
            entities: HashMap::new(),
            active_combats: Vec::new(),
        }
    }
}

/// Manages the location of all entities in the world.
#[derive(Clone, Debug)]
struct LocationManager {
    locations: Arc<RwLock<HashMap<usize, usize>>>, // Key: entity_id, Value: room_id
}

impl LocationManager {
    fn new() -> Self {
        Self { locations: Arc::new(RwLock::new(HashMap::new())) }
    }
    async fn get_room_id(&self, entity_id: &usize) -> Option<usize> {
        self.locations.read().await.get(entity_id).cloned()
    }
    async fn set_room_id(&self, entity_id: usize, room_id: usize) {
        self.locations.write().await.insert(entity_id, room_id);
    }
    async fn remove_location(&self, entity_id: &usize) {
        self.locations.write().await.remove(entity_id);
    }
}

//---------- Application State ----------//

#[derive(Clone)]
struct AppState {
    world: Arc<RwLock<World>>,
    next_entity_id: Arc<AtomicUsize>,
    player_database: Arc<RwLock<HashMap<String, PersistentPlayerData>>>,
    locations: LocationManager,
}

const SPAWN_ROOM_ID: usize = 0;

//---------- Main Application & Game Loop ----------//

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "server=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let next_entity_id = Arc::new(AtomicUsize::new(1));
    let mut world = World::new();
    let locations = LocationManager::new();

    let bandit_id = next_entity_id.fetch_add(1, Ordering::Relaxed);
    let bandit = Npc::new_bandit(bandit_id, 1); 
    world.entities.insert(bandit_id, Box::new(bandit));
    locations.set_room_id(bandit_id, 1).await; 

    let app_state = AppState {
        world: Arc::new(RwLock::new(world)),
        next_entity_id,
        player_database: Arc::new(RwLock::new(HashMap::new())),
        locations,
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
    let mut interval = tokio::time::interval(Duration::from_secs(2));
    loop {
        interval.tick().await;
        let mut world = state.world.write().await;

        // --- Regenerate Player Stats ---
        for entity in world.entities.values_mut() {
            if let Some(player) = entity.as_player() {
                let mut p_state = player.state.clone();
                if p_state.derived.hp < p_state.derived.max_hp {
                    p_state.derived.hp = (p_state.derived.hp + 1).min(p_state.derived.max_hp);
                }
                if p_state.derived.mp < p_state.derived.max_mp {
                    p_state.derived.mp = (p_state.derived.mp + 1).min(p_state.derived.max_mp);
                }
            }
        }

        // --- Process Combat ---
        let mut dead_entities = Vec::new();
        let combat_pairs = world.active_combats.clone();

        for (attacker_id, defender_id) in combat_pairs {
            if dead_entities.contains(&attacker_id) || dead_entities.contains(&defender_id) {
                continue;
            }

            if let Some(mut attacker_box) = world.entities.remove(&attacker_id) {
                if let Some(defender_box) = world.entities.get_mut(&defender_id) {
                    let attacker = attacker_box.as_mut().as_mut();
                    let defender = defender_box.as_mut().as_mut();

                    let result = combat::resolve_attack_round(attacker, defender);

                    if result.is_hit {
                        // [PERFORMANCE] Send real-time state update after a hit.
                        // For the current 1v1 combat, this is perfectly performant.
                        // TODO: For a large-scale MMO, this should be refactored into a "throttled" or "batched" 
                        // update system. Instead of sending on every hit, we would collect all state changes 
                        // over a ~250ms window and send only the latest state to the client.
                        if let Some(player) = attacker_box.as_ref().as_player() {
                            let sender = player.sender.clone();
                            let stats = player.get_state().derived.clone();
                            tokio::spawn(async move {
                                sender.send(ServerMessage::DerivedStatsUpdate { state: stats }).await.ok();
                            });
                        }
                        
                        if let Some(player) = defender_box.as_ref().as_player() {
                            let sender = player.sender.clone();
                            let stats = player.get_state().derived.clone();
                            tokio::spawn(async move {
                                sender.send(ServerMessage::DerivedStatsUpdate { state: stats }).await.ok();
                            });
                        }
                    }

                    if result.defender_is_dead {
                        dead_entities.push(defender_id);
                    }
                }
                world.entities.insert(attacker_id, attacker_box);
            }
        }

        world.active_combats.retain(|(aid, did)| !dead_entities.contains(aid) && !dead_entities.contains(did));

        // Remove dead entities
        for dead_id in dead_entities {
            if let Some(dead_entity) = world.entities.remove(&dead_id) {
                let locations = state.locations.clone();
                tokio::spawn(async move { locations.remove_location(&dead_id).await; });
                tracing::debug!("Entity {} ({}) removed from world.", dead_id, dead_entity.as_ref().as_ref().get_name());
            }
        }
    }
}

//---------- WebSocket Handling ----------//

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (ws_sender, mut ws_receiver) = socket.split();
    let (tx, rx) = mpsc::channel(100);
    tokio::spawn(forward_game_updates_to_ws(ws_sender, rx));

    let mut player_id: Option<usize> = None;
    let mut username: Option<String> = None;
    
    if let Some(Ok(Message::Text(text))) = ws_receiver.next().await {
        if let Ok(ClientMessage::Login { username: u }) = serde_json::from_str::<ClientMessage>(&text) {
             let id = state.next_entity_id.fetch_add(1, Ordering::Relaxed);
             player_id = Some(id);
             username = Some(u.clone());
             
             let (initial_state, initial_room_id, is_new) = {
                 let db = state.player_database.read().await;
                 match db.get(&u) {
                     Some(data) => (data.state.clone(), data.room_id, false),
                     None => (PlayerState::new_player_default(), SPAWN_ROOM_ID, true),
                 }
             };

            let new_player = Player { id, username: u.clone(), state: initial_state, sender: tx.clone() };
            state.world.write().await.entities.insert(id, Box::new(new_player));
            state.locations.set_room_id(id, initial_room_id).await;

            let world = state.world.read().await;
            let room = world.rooms.get(&initial_room_id).unwrap();
            let welcome_msg = if is_new { format!("Welcome, {}!\n{}", u, room.get_description()) } else { format!("Welcome back, {}!\n{}", u, room.get_description()) };
            let _ = tx.send(ServerMessage::GameMessage { content: welcome_msg }).await;
            if let Some(p) = world.entities.get(&id) {
                 let _ = tx.send(ServerMessage::FullState { state: p.as_ref().as_ref().get_state().clone() }).await;
            }
        }
    }

    if let (Some(id), Some(uname)) = (player_id, username.clone()) {
        while let Some(Ok(Message::Text(text))) = ws_receiver.next().await {
            if let Ok(ClientMessage::Command { command }) = serde_json::from_str::<ClientMessage>(&text) {
                let response = process_command(id, &command, &state).await;
                let _ = tx.send(ServerMessage::GameMessage { content: response }).await;
            }
        }

        if let Some(entity) = state.world.write().await.entities.remove(&id) {
            let mut db = state.player_database.write().await;
            let room_id = state.locations.get_room_id(&id).await.unwrap_or(SPAWN_ROOM_ID);
            db.insert(uname.clone(), PersistentPlayerData { state: entity.as_ref().as_ref().get_state().clone(), room_id });
            tracing::info!("Player {} ({}) disconnected and data saved.", id, uname);
         }
         state.locations.remove_location(&id).await;
    }
}

async fn forward_game_updates_to_ws(mut ws_sender: SplitSink<WebSocket, Message>, mut rx: mpsc::Receiver<ServerMessage>) {
    while let Some(msg) = rx.recv().await {
        if let Ok(json_msg) = serde_json::to_string(&msg) {
            if ws_sender.send(Message::Text(json_msg)).await.is_err() {
                break;
            }
        }
    }
}

//---------- Command Processing ----------//

async fn process_command(player_id: usize, command_str: &str, state: &AppState) -> String {
    let parts: Vec<&str> = command_str.trim().split_whitespace().collect();
    let command = parts.get(0).map_or("", |s| *s).to_lowercase();

    let player_room_id = match state.locations.get_room_id(&player_id).await {
        Some(id) => id,
        None => return "Error: Your player has no location in the world.".to_string(),
    };

    match command.as_str() {
        "look" => {
            let world = state.world.read().await;
            let locations = state.locations.locations.read().await;
            let room = world.rooms.get(&player_room_id).unwrap();
            let exits = room.get_exits().keys().map(|s| s.as_str()).collect::<Vec<_>>().join(", ");

            let mut others_in_room = Vec::new();
            for (id, entity) in &world.entities {
                if *id != player_id && locations.get(id) == Some(&player_room_id) {
                    others_in_room.push(entity.as_ref().as_ref().get_name().to_string());
                }
            }
            let others_desc = if others_in_room.is_empty() { "".to_string() } else { format!("\nYou also see: {}", others_in_room.join(", ")) };

            format!("{} - {}\nExits: {}{}", room.get_name(), room.get_description(), exits, others_desc)
        }
        "go" => {
            let direction = parts.get(1).map_or("", |s| *s);
            if direction.is_empty() { return "Go where?".to_string(); }
            
            let world = state.world.read().await;
            let room = world.rooms.get(&player_room_id).unwrap();

            if let Some(next_room_id) = room.get_exits().get(direction) {
                state.locations.set_room_id(player_id, *next_room_id).await;
                let next_room = world.rooms.get(next_room_id).unwrap();
                format!("You move {}.\n{}\nExits: {}", direction, next_room.get_description(), next_room.get_exits().keys().map(|s| s.as_str()).collect::<Vec<_>>().join(", "))
            } else {
                "You can't go that way.".to_string()
            }
        }
        "kill" => {
            let target_name = parts.get(1).map_or("", |s| *s);
            if target_name.is_empty() { return "Kill what?".to_string(); }
            
            let mut world = state.world.write().await;
            let locations = state.locations.locations.read().await;
            
            let mut target_id: Option<usize> = None;
            for (id, entity) in &world.entities {
                if locations.get(id) == Some(&player_room_id) && entity.as_ref().as_ref().get_name().eq_ignore_ascii_case(target_name) {
                    target_id = Some(*id);
                    break;
                }
            }

            if let Some(id) = target_id {
                if id == player_id { return "You can't fight yourself.".to_string(); }
                world.active_combats.push((player_id, id));
                world.active_combats.push((id, player_id));
                format!("You attack the {}!", target_name)
            } else {
                format!("There is no one named '{}' here.", target_name)
            }
        }
        "status" | "score" => {
            let world = state.world.read().await;
            if let Some(player_entity) = world.entities.get(&player_id) {
                let combatant_state = player_entity.as_ref().as_ref().get_state();
                 format!(
                    "--- Player Status ---\nLevel: {level} | Exp: {exp}/{exp_to_level}\nHP: {hp}/{max_hp} | MP: {mp}/{max_mp}\n\nStr: {str} | Agi: {agi} | Con: {con} | Comp: {comp}",
                    level = combatant_state.progression.level, exp = combatant_state.progression.experience, exp_to_level = 100, // Placeholder
                    hp = combatant_state.derived.hp, max_hp = combatant_state.derived.max_hp, mp = combatant_state.derived.mp, max_mp = combatant_state.derived.max_mp,
                    str = combatant_state.base.strength, agi = combatant_state.base.agility, con = combatant_state.base.constitution, comp = combatant_state.base.comprehension
                )
            } else {
                 "Could not find your player data.".to_string()
            }
        }
        "help" => {
            "Commands: look, go [dir], kill [target], status, help".to_string()
        }
        _ => "Unknown command.".to_string(),
    }
}
