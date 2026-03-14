
#[cfg(test)]
mod tests;
pub mod command;
pub mod npc;
pub mod world;
pub mod world_model;
pub mod combat;
pub mod ui;
pub mod game_loop;
pub mod commands;

#[cfg(feature = "gotify")]
pub mod gotify;

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
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::SystemTime as StdSystemTime,
};
use tokio::sync::mpsc;
use tower_http::services::ServeDir;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use colored::*;

use crate::command::{parse, Command};
use crate::npc::Npc;
use crate::world::world_player::{Player, PlayerQuestStatus};
use crate::world::world_state::WorldState;

#[derive(Deserialize)]
#[serde(tag = "type")]
enum ClientMessage {
    Login { user_id: String },
    Command { command: String },
}

#[derive(Serialize, Clone)]
#[serde(tag = "type")]
pub enum ServerMessage {
    Description { payload: String },
    Info { payload: String },
    Error { payload: String },
}

pub struct AppState {
    pub world_state: WorldState,
    pub player_sessions: Mutex<HashMap<u64, PlayerSession>>,
}

#[derive(Clone)]
pub struct PlayerSession {
    player: Player,
    pub user_id: Option<String>,
    sender: mpsc::Sender<Message>,
}

fn broadcast_room_movement(state: &Arc<AppState>, from_room: &str, to_room: &str, user_name: &Option<String>) {
    let user_name = match user_name {
        Some(name) => name.clone(),
        None => return,
    };

    let rooms_to_notify = vec![from_room, to_room];
    
    for room_id in rooms_to_notify {
        let players_in_room: Vec<(u64, mpsc::Sender<Message>)> = {
            let data = state.world_state.dynamic_data.lock().unwrap();
            let player_ids: Vec<u64> = data.players.iter()
                .filter(|(id, loc)| loc.room_id == room_id && loc.user_name.as_ref() != Some(&user_name))
                .map(|(id, _)| *id)
                .collect();
            drop(data);
            
            let sessions = state.player_sessions.lock().unwrap();
            player_ids.iter()
                .filter_map(|id| sessions.get(id).map(|s| (s.player.id, s.sender.clone())))
                .collect()
        };

        let message = if room_id == from_room {
            ServerMessage::Info { payload: format!("{} 离开了。", user_name.yellow()) }
        } else {
            ServerMessage::Info { payload: format!("{} 走进了房间。", user_name.cyan()) }
        };

        if let Ok(json) = serde_json::to_string(&message) {
            for (_, sender) in players_in_room {
                let _ = sender.try_send(axum::extract::ws::Message::Text(json.clone()));
            }
        }
    }
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
        game_loop::game_loop(game_loop_state).await;
    });

    let static_dir = env::var("STATIC_DIR").unwrap_or_else(|_| "client/dist".to_string());
    info!("Serving static files from: {}", static_dir);

    let app = Router::new()
        .route("/ws", get(websocket_handler))
        .nest_service("/", ServeDir::new(static_dir))
        .with_state(app_state);

    let port = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap_or(3000);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("listening on {}", addr);
    axum::serve(tokio::net::TcpListener::bind(&addr).await.unwrap(), app.into_make_service())
        .await
        .unwrap();
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| websocket(socket, state))
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
    state.world_state.move_player_to_room(player_id, &starting_room, None);
    
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
                            let world = &state.world_state;
                            let mut tutorial_given = false;

                            let maybe_sender = {
                                let mut sessions = state.player_sessions.lock().unwrap();
                                if let Some(session) = sessions.get_mut(&player_id) {
                                    session.user_id = Some(user_id.clone());
                                    session.player.name = user_id.clone();
                                    session.player.last_input_time = StdSystemTime::now().duration_since(StdSystemTime::UNIX_EPOCH).unwrap().as_secs();
                                    
                                    state.world_state.update_player_name(player_id, user_id.clone());
                                    tracing::info!("[LOGIN] Player {} logged in as {}", player_id, user_id);

                                    if session.player.active_quests.is_empty() && session.player.completed_quests.is_empty() {
                                        if world.static_data.quests.contains_key("tutorial_1") {
                                            session.player.active_quests.push(PlayerQuestStatus {
                                                quest_id: "tutorial_1".to_string(),
                                                current_step: 0,
                                                is_completed: false,
                                                kill_counts: HashMap::new(),
                                            });
                                            tutorial_given = true;
                                        }
                                    }

                                    Some(session.sender.clone())
                                } else {
                                    None
                                }
                            };

                            if let Some(session_sender) = maybe_sender {
                                let mut welcome_content = format!("{}

{}", ui::build_welcome_message(), ui::get_full_room_description("genesis_altar", world, Vec::new(), Vec::new(), Vec::new()));
                                
                                if tutorial_given {
                                    welcome_content.push_str(&format!("
{}", "[任务提示] 你收到了一项新任务：初入凡尘。输入 'qs' 可随时查看任务进度。".yellow().bold()));
                                }

                                let welcome_msg = ServerMessage::Description { payload: welcome_content };
                                if let Ok(json_str) = serde_json::to_string(&welcome_msg) {
                                    let _ = session_sender.send(Message::Text(json_str)).await;
                                }
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
                    Err(_) => {}
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
    use crate::ui::get_full_room_description;

    let mut messages_to_send = Vec::new();

    if matches!(command, Command::Who) {
        tracing::info!("[WHO] Player {} requested who list", player_id);
        messages_to_send.push(commands::handle_who(&state));
    } else {
        let mut session_lock = state.player_sessions.lock().unwrap();
        let session = match session_lock.get_mut(&player_id) {
            Some(s) => s,
            None => return,
        };

        if session.user_id.is_none() {
            messages_to_send.push(ServerMessage::Error { payload: "You must login first.".to_string() });
        } else {
            session.player.last_input_time = StdSystemTime::now().duration_since(StdSystemTime::UNIX_EPOCH).unwrap().as_secs();
            let world = &state.world_state;

            if let Some(current_room_id) = world.get_player_room_id(player_id) {
                let current_room_id_str = current_room_id.clone();
                if let Some(current_room) = world.get_room(&current_room_id_str) {
                    
                    let block_resting = match command {
                        Command::Look | Command::Score | Command::Quest | Command::Inventory | Command::Rest | Command::Help => false,
                        _ => session.player.is_resting,
                    };

                    if block_resting {
                        messages_to_send.push(ServerMessage::Error { payload: "你正在休息中，无法执行此操作。".to_string() });
                    } else {
                        match command {
                            Command::Help => {
                                messages_to_send.push(commands::handle_help());
                            }
                            Command::Rest => {
                                messages_to_send.push(commands::handle_rest(&mut session.player));
                            }
                            Command::Attack { target } => {
                                let user_name = session.user_id.clone().unwrap_or_default();
                                if let Some(msg) = commands::handle_attack(
                                    &mut session.player,
                                    &target,
                                    &current_room_id_str,
                                    world,
                                    player_id,
                                    &user_name,
                                ) {
                                    messages_to_send.push(msg);
                                } else {
                                    messages_to_send.push(ServerMessage::Error { payload: format!("你在这没看到 {}。", target) });
                                }
                            }
                            Command::Kill { target } => {
                                let user_name = session.user_id.clone().unwrap_or_default();
                                if let Some(msg) = commands::handle_kill(
                                    &mut session.player,
                                    &target,
                                    &current_room_id_str,
                                    world,
                                    player_id,
                                    &user_name,
                                ) {
                                    messages_to_send.push(msg);
                                } else {
                                    messages_to_send.push(ServerMessage::Error { payload: format!("你在这没看到 {}。", target) });
                                }
                            }
                            Command::Cast { skill, target } => {
                                if let Some(msg) = commands::handle_cast(
                                    &mut session.player,
                                    &skill,
                                    target.as_deref(),
                                    &current_room_id_str,
                                    world,
                                ) {
                                    messages_to_send.push(msg);
                                } else {
                                    messages_to_send.push(ServerMessage::Error { payload: format!("没有找到技能: {}", skill) });
                                }
                            }
                            Command::Work => {
                                let msgs = commands::handle_work(&mut session.player, &current_room_id_str);
                                messages_to_send.extend(msgs);
                            }
                            Command::Go { direction } => {
                                if let Some(cs) = &session.player.combat_state {
                                    messages_to_send.push(ServerMessage::Error { 
                                        payload: format!("你正在和{}战斗，不能移动！", cs.target_name) 
                                    });
                                } else if let Some(next_room_id) = current_room.exits.get(&direction) {
                                    if !session.player.consume_stamina(1) {
                                        messages_to_send.push(ServerMessage::Error { payload: "你太累了，走不动了。".to_string() });
                                    } else {
                                        let next_room_id_str = next_room_id.clone();
                                        let user_name = session.user_id.clone();
                                        let from_room_id = current_room_id_str.clone();
                                        tracing::info!("[MOVE] Player {} moved from {} to {}", player_id, from_room_id, next_room_id_str);
                                        world.move_player_to_room(player_id, &next_room_id_str, user_name.clone());
                                        
                                        let mut quest_updates = Vec::new();
                                        for status in &mut session.player.active_quests {
                                            if let Some(quest) = world.static_data.quests.get(&status.quest_id) {
                                                if let Some(step) = quest.steps.get(status.current_step as usize) {
                                                    if step.step_type == "move" && step.target_id == next_room_id_str {
                                                        status.current_step += 1;
                                                        quest_updates.push((quest.name.clone(), status.current_step as usize == quest.steps.len(), quest.rewards.clone()));
                                                    }
                                                }
                                            }
                                        }

                                        let mut payload = get_full_room_description(&next_room_id_str, world, Vec::new(), Vec::new(), Vec::new());
                                        for (name, is_finished, rewards) in quest_updates {
                                            if is_finished {
                                                payload.push_str(&format!("
{}", format!("[任务完成] {}", name).yellow().bold()));
                                                let reward_text = session.player.grant_reward(&rewards);
                                                messages_to_send.push(ServerMessage::Info { payload: reward_text });
                                            } else {
                                                payload.push_str(&format!("
{}", format!("[任务更新] {}", name).green().bold()));
                                            }
                                        }
                                        session.player.active_quests.retain(|q| {
                                            if let Some(qd) = world.static_data.quests.get(&q.quest_id) {
                                                if qd.quest_type == "kill" {
                                                    if q.is_completed {
                                                        session.player.completed_quests.insert(q.quest_id.clone());
                                                        return false;
                                                    }
                                                } else if q.current_step as usize >= qd.steps.len() {
                                                    session.player.completed_quests.insert(q.quest_id.clone());
                                                    return false;
                                                }
                                            }
                                            true
                                        });

                                        messages_to_send.push(ServerMessage::Description { payload });
                                        
                                        drop(session_lock);
                                        broadcast_room_movement(&state, &from_room_id, &next_room_id_str, &user_name);
                                        session_lock = state.player_sessions.lock().unwrap();
                                        if let Some(session) = session_lock.get_mut(&player_id) {
                                            session.player.last_input_time = StdSystemTime::now().duration_since(StdSystemTime::UNIX_EPOCH).unwrap().as_secs();
                                        }
                                    }
                                } else {
                                    messages_to_send.push(ServerMessage::Error { payload: "You can't go that way.".to_string() });
                                }
                            }
                            Command::Talk { target } => {
                                let npcs = world.get_npcs_in_room(&current_room_id_str);
                                if let Some(npc) = npcs.iter().find(|n| n.name == target || n.prototype_id.to_string() == target) {
                                    let mut dialog_id = None;
                                    let mut quest_finished = false;
                                    let mut quest_reward = None;
                                    let mut quest_name = String::new();
                                    let mut finished_quest_id = String::new();

                                    for status in &mut session.player.active_quests {
                                        if let Some(quest) = world.static_data.quests.get(&status.quest_id) {
                                            if let Some(step) = quest.steps.get(status.current_step as usize) {
                                                if step.step_type == "talk" && step.target_id == npc.prototype_id.to_string() {
                                                    dialog_id = step.dialog_id.clone();
                                                    status.current_step += 1;
                                                    quest_name = quest.name.clone();
                                                    if status.current_step as usize == quest.steps.len() {
                                                        quest_finished = true;
                                                        quest_reward = Some(quest.rewards.clone());
                                                        finished_quest_id = status.quest_id.clone();
                                                    }
                                                    break;
                                                }
                                            }
                                        }
                                    }

                                    let final_dialog = dialog_id.unwrap_or_else(|| {
                                        npc.dialog_id.clone().unwrap_or_else(|| "default_greet".to_string())
                                    });

                                    let mut payload = format!("{}: {}", npc.name, final_dialog);
                                    if !quest_name.is_empty() {
                                        if quest_finished {
                                            payload.push_str(&format!("
{}", format!("[任务完成] {}", quest_name).yellow().bold()));
                                            if let Some(r) = quest_reward {
                                                let reward_text = session.player.grant_reward(&r);
                                                messages_to_send.push(ServerMessage::Info { payload: reward_text });
                                            }
                                            session.player.active_quests.retain(|q| {
                                                if let Some(qd) = world.static_data.quests.get(&q.quest_id) {
                                                    if qd.quest_type == "kill" {
                                                        if q.is_completed {
                                                            session.player.completed_quests.insert(q.quest_id.clone());
                                                            return false;
                                                        }
                                                    } else if q.current_step as usize >= qd.steps.len() {
                                                        session.player.completed_quests.insert(q.quest_id.clone());
                                                        return false;
                                                    }
                                                }
                                                true
                                            });

                                            if finished_quest_id == "tutorial_1" && npc.prototype_id == 1002 {
                                                if world.static_data.quests.contains_key("q102") && !session.player.completed_quests.contains("q102") {
                                                    session.player.active_quests.push(PlayerQuestStatus {
                                                        quest_id: "q102".to_string(),
                                                        current_step: 0,
                                                        is_completed: false,
                                                        kill_counts: HashMap::new(),
                                                    });
                                                    payload.push_str(&format!("
{}", "[任务接取] 老村长交给了你一个新的任务：勤能补拙。输入 'qs' 查看详情。".yellow().bold()));
                                                }
                                            }
                                        } else {
                                            payload.push_str(&format!("
{}", format!("[任务更新] {}", quest_name).green().bold()));
                                        }
                                    } else {
                                        if npc.prototype_id == 1002 && session.player.completed_quests.contains("tutorial_1") && 
                                           !session.player.completed_quests.contains("q102") && 
                                           !session.player.active_quests.iter().any(|q| q.quest_id == "q102") {
                                            if world.static_data.quests.contains_key("q102") {
                                                 session.player.active_quests.push(PlayerQuestStatus {
                                                    quest_id: "q102".to_string(),
                                                    current_step: 0,
                                                    is_completed: false,
                                                    kill_counts: HashMap::new(),
                                                });
                                                payload.push_str(&format!("
{}", "[任务接取] 老村长交给了你一个新的任务：勤能补拙。输入 'qs' 查看详情。".yellow().bold()));
                                            }
                                        }
                                    }
                                    messages_to_send.push(ServerMessage::Description { payload });
                                } else {
                                    messages_to_send.push(ServerMessage::Error { payload: format!("这里没有 {}。", target) });
                                }
                            }
                            Command::Accept { quest_id } => {
                                if let Some(msg) = commands::handle_accept(&mut session.player, &quest_id, &current_room_id_str, world) {
                                    messages_to_send.push(msg);
                                } else {
                                    messages_to_send.push(ServerMessage::Error { payload: "没有找到这个任务。".to_string() });
                                }
                            }
                            Command::Get { item } => {
                                if let Some(msg) = commands::handle_get(&mut session.player, &item, &current_room_id_str, world) {
                                    messages_to_send.push(msg);
                                } else {
                                    messages_to_send.push(ServerMessage::Error { payload: format!("这里没有 {}。", item) });
                                }
                            }
                            Command::Inventory => {
                                messages_to_send.push(commands::handle_inventory(&session.player, world));
                            }
                            Command::Quest => {
                                messages_to_send.push(commands::handle_quest(&session.player, world));
                            }
                            Command::Look => {
                                tracing::info!("[LOOK] Player {} looking at room {}", player_id, current_room_id_str);
                                let msg = commands::handle_look(player_id, &current_room_id_str, world, &session.player);
                                messages_to_send.push(msg);
                            }
                            Command::Score => {
                                messages_to_send.push(commands::handle_score(&session.player, &world.static_data.config));
                            }
                            Command::Unknown(ref cmd) if cmd == "heartbeat" => return,
                            Command::Unknown(cmd) => messages_to_send.push(ServerMessage::Error { payload: format!("Unknown command: {}", cmd) }),
                            _ => messages_to_send.push(ServerMessage::Info { payload: "Command not yet implemented.".to_string() }),
                        }
                    }
                } else {
                    tracing::error!("Player {} is in an invalid room: {}", player_id, current_room_id);
                    messages_to_send.push(ServerMessage::Error { payload: "Your location is corrupted. Please reconnect.".to_string() });
                }
            } else {
                tracing::error!("Could not find location for player {}", player_id);
                messages_to_send.push(ServerMessage::Error { payload: "Critical error: Your location is unknown.".to_string() });
            }
        }
    }

    for server_msg in messages_to_send {
        if let Ok(json_str) = serde_json::to_string(&server_msg) {
            let _ = sender.send(Message::Text(json_str)).await;
        }
    }
}

fn generate_who_list(state: &Arc<AppState>, use_color: bool) -> String {
    crate::ui::generate_who_list(state, use_color)
}
