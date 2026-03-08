#[cfg(test)]
mod tests;
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
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::SystemTime as StdSystemTime,
};
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use tower_http::services::ServeDir;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::command::{parse, Command};
use crate::world::player::{Player, PlayerQuestStatus};
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
    let mut tick_counter: u64 = 0;
    const RECOVERY_TICK_INTERVAL: u64 = 10;

    loop {
        sleep(Duration::from_secs(1)).await;
        tick_counter += 1;

        if tick_counter % RECOVERY_TICK_INTERVAL == 0 {
            let now = StdSystemTime::now()
                .duration_since(StdSystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs();

            let mut sessions = app_state.player_sessions.lock().unwrap();
            for session in sessions.values_mut() {
                if session.user_id.is_none() {
                    continue;
                }

                session.player.on_heartbeat_recovery();

                let room_id = app_state.world_state.get_player_room_id(session.player.id).unwrap_or_default();
                if (room_id == "bamboo_forest" || room_id == "spirit_spring") && now - session.player.last_input_time > 30 {
                    let hint = ServerMessage::Info { 
                        payload: "\x1b[1;36m[提示]：你现在应该尝试输入 work 指令来进行伐木。记得随时输入 score 查看你的体力值。\x1b[0m".to_string() 
                    };
                    if let Ok(json) = serde_json::to_string(&hint) {
                        let _ = session.sender.try_send(Message::Text(json));
                    }
                    session.player.last_input_time = now;
                }
            }
        }
    }
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| websocket(socket, state))
}

fn get_full_room_description(room_id: &str, world_state: &WorldState) -> String {
    if let Some(room) = world_state.get_room(room_id) {
        let mut full_desc = format!("{}\n{}", room.name, room.description);

        let npcs_in_room = world_state.get_npcs_in_room(room_id);
        if !npcs_in_room.is_empty() {
            let npc_names: Vec<String> = npcs_in_room
                .iter()
                .map(|npc| format!("\x1b[32m{}\x1b[0m", npc.name))
                .collect();
            full_desc.push_str(&format!("\n● {}", npc_names.join(", ")));
        }

        let room_items = world_state.get_items_in_room(room_id);
        if !room_items.is_empty() {
            let item_names: Vec<String> = room_items
                .iter()
                .filter_map(|id| world_state.static_data.item_prototypes.get(id))
                .map(|item| item.name.clone())
                .collect();
            if !item_names.is_empty() {
                full_desc.push_str(&format!("\n{}", item_names.join(", ")));
            }
        }

        if !room.exits.is_empty() {
            let exit_keys: Vec<String> = room.exits.keys().cloned().collect();
            full_desc.push_str(&format!("\n出口: [{}]", exit_keys.join(", ")));
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
                                    session.player.name = user_id;
                                    session.player.last_input_time = StdSystemTime::now().duration_since(StdSystemTime::UNIX_EPOCH).unwrap().as_secs();

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
                                let mut welcome_content = format!("{}\n\n{}", world.static_data.config.welcome_message, get_full_room_description("genesis_altar", world));
                                
                                if tutorial_given {
                                    welcome_content.push_str("\n\x1b[1;33m[任务提示] 你收到了一项新任务：初入凡尘。输入 'qs' 可随时查看任务进度。\x1b[0m");
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
    let mut messages_to_send = Vec::new();

    {
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
                        Command::Look | Command::Score | Command::Quest | Command::Inventory | Command::Rest => false,
                        _ => session.player.is_resting,
                    };

                    if block_resting {
                        messages_to_send.push(ServerMessage::Error { payload: "你正在休息中，无法执行此操作。".to_string() });
                    } else {
                        match command {
                            Command::Rest => {
                                session.player.is_resting = !session.player.is_resting;
                                if session.player.is_resting {
                                    messages_to_send.push(ServerMessage::Info { payload: "你开始原地休息，逐渐恢复精力。".to_string() });
                                } else {
                                    messages_to_send.push(ServerMessage::Info { payload: "你站了起来，感觉精力充沛了一些。".to_string() });
                                }
                            }
                            Command::Attack { target } => {
                                let npcs = world.get_npcs_in_room(&current_room_id_str);
                                if let Some(npc) = npcs.iter().find(|n| n.name == target || n.prototype_id.to_string() == target) {
                                    let is_monster = if let Some(proto) = world.static_data.npc_prototypes.get(&npc.prototype_id) {
                                        proto.ai == "monster" || !proto.flags.contains(&"friendly".to_string())
                                    } else {
                                        true
                                    };

                                    if is_monster {
                                        let combat_msg = format!("你对着 {} 发起猛攻，几个回合后将其击败了！", npc.name);
                                        messages_to_send.push(ServerMessage::Description { payload: combat_msg });

                                        // Update quest progress
                                        let quest_msg = session.player.on_kill(&npc.prototype_id.to_string(), &world.static_data.quests);
                                        if !quest_msg.is_empty() {
                                            messages_to_send.push(ServerMessage::Info { payload: quest_msg });
                                        }

                                        // Check for completed kill quests and grant rewards immediately
                                        let completed_ids: Vec<String> = session.player.active_quests.iter()
                                            .filter(|status| status.is_completed)
                                            .filter_map(|status| {
                                                world.static_data.quests.get(&status.quest_id)
                                                    .filter(|quest| quest.quest_type == "kill")
                                                    .map(|_| status.quest_id.clone())
                                            })
                                            .collect();
                                        
                                        for id in &completed_ids {
                                            if let Some(quest) = world.static_data.quests.get(id as &String) {
                                                let reward_msg = session.player.grant_reward(&quest.rewards);
                                                messages_to_send.push(ServerMessage::Info { payload: reward_msg });
                                            }
                                            session.player.completed_quests.insert(id.clone());
                                        }
                                        
                                        session.player.active_quests.retain(|q| !completed_ids.contains(&q.quest_id));

                                        // Remove NPC instance
                                        let mut dynamic_data = world.dynamic_data.lock().unwrap();
                                        dynamic_data.npcs.remove(&npc.instance_id);
                                    } else {
                                        messages_to_send.push(ServerMessage::Error { payload: format!("{} 看起来很友善，你下不了手。", npc.name) });
                                    }
                                } else {
                                    messages_to_send.push(ServerMessage::Error { payload: format!("你在这没看到 {}。", target) });
                                }
                            }
                            Command::Work => {
                                if current_room_id_str != "bamboo_forest" && current_room_id_str != "spirit_spring" {
                                    messages_to_send.push(ServerMessage::Error { payload: "这里似乎没有什么值得你劳作的地方，换个环境试试？".to_string() });
                                } else if !session.player.consume_stamina(15) {
                                    messages_to_send.push(ServerMessage::Error { payload: "你已经筋疲力尽了，稍微休息（rest）一下吧。".to_string() });
                                } else {
                                    session.player.wallet.shell += 20;
                                    session.player.exp += 5;
                                    session.player.potential += 2;

                                    let pool = [
                                        "“你抡起斧头劈向枯木，震得虎口生疼，但隐约间你捕捉到了风的律动。”",
                                        "“汗水顺着脸颊流下，你进入了一种奇妙的节奏，呼吸逐渐与竹林的沙沙声同步。”",
                                        "“每一次挥砍都带起片片竹叶，你感到体内有一丝微弱的气流正随着动作缓缓升起。”"
                                    ];
                                    let mut rng = rand::thread_rng();
                                    let msg = pool.choose(&mut rng).unwrap().to_string();
                                    messages_to_send.push(ServerMessage::Description { payload: msg });
                                    messages_to_send.push(ServerMessage::Info { payload: "\x1b[1;32m获得奖励：灵贝+20，修为+5，潜能+2\x1b[0m".to_string() });

                                    let mut q102_finished = false;
                                    for status in &mut session.player.active_quests {
                                        if status.quest_id == "q102" && status.current_step == 2 {
                                            let count = session.player.quest_counts.entry("q102_work".to_string()).or_insert(0);
                                            *count += 1;
                                            if *count >= 5 {
                                                status.current_step += 1;
                                                q102_finished = true;
                                            }
                                        }
                                    }
                                    if q102_finished {
                                        messages_to_send.push(ServerMessage::Description { payload: "\n\x1b[1;35m“【机缘】随着最后一斧劈下，你感到一股清凉的气流顺着指尖流向全身。你对天地的感悟达到了新的高度！请回广场向村长报告。”\x1b[0m".to_string() });
                                    }
                                }
                            }
                            Command::Go { direction } => {
                                if let Some(next_room_id) = current_room.exits.get(&direction) {
                                    if !session.player.consume_stamina(1) {
                                        messages_to_send.push(ServerMessage::Error { payload: "你太累了，走不动了。".to_string() });
                                    } else {
                                        let next_room_id_str = next_room_id.clone();
                                        world.move_player_to_room(player_id, &next_room_id_str);
                                        
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

                                        let mut payload = get_full_room_description(&next_room_id_str, world);
                                        for (name, is_finished, rewards) in quest_updates {
                                            if is_finished {
                                                payload.push_str(&format!("\n\x1b[1;33m[任务完成] {}\x1b[0m", name));
                                                let reward_text = session.player.grant_reward(&rewards);
                                                messages_to_send.push(ServerMessage::Info { payload: reward_text });
                                            } else {
                                                payload.push_str(&format!("\n\x1b[1;32m[任务更新] {}\x1b[0m", name));
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
                                            payload.push_str(&format!("\n\x1b[1;33m[任务完成] {}\x1b[0m", quest_name));
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

                                            // Check for next quest: q102 after tutorial_1 if talking to chief (1002)
                                            if finished_quest_id == "tutorial_1" && npc.prototype_id == 1002 {
                                                if world.static_data.quests.contains_key("q102") && !session.player.completed_quests.contains("q102") {
                                                    session.player.active_quests.push(PlayerQuestStatus {
                                                        quest_id: "q102".to_string(),
                                                        current_step: 0,
                                                        is_completed: false,
                                                        kill_counts: HashMap::new(),
                                                    });
                                                    payload.push_str("\n\x1b[1;33m[任务接取] 老村长交给了你一个新的任务：勤能补拙。输入 'qs' 查看详情。\x1b[0m");
                                                }
                                            }
                                        } else {
                                            payload.push_str(&format!("\n\x1b[1;32m[任务更新] {}\x1b[0m", quest_name));
                                        }
                                    } else {
                                        // NPC doesn't have an active quest step for player, check if they can start a new quest
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
                                                payload.push_str("\n\x1b[1;33m[任务接取] 老村长交给了你一个新的任务：勤能补拙。输入 'qs' 查看详情。\x1b[0m");
                                            }
                                        }
                                    }
                                    messages_to_send.push(ServerMessage::Description { payload });
                                } else {
                                    messages_to_send.push(ServerMessage::Error { payload: format!("这里没有 {}。", target) });
                                }
                            }
                            Command::Accept { quest_id } => {
                                let npcs = world.get_npcs_in_room(&current_room_id_str);
                                let has_board = npcs.iter().any(|n| n.prototype_id == 2000);
                                
                                if has_board {
                                    if let Some(quest) = world.static_data.quests.get(&quest_id) {
                                        if session.player.completed_quests.contains(&quest_id) {
                                            messages_to_send.push(ServerMessage::Error { payload: "你已经完成了这个任务，不能重复接取。".to_string() });
                                        } else if session.player.active_quests.iter().any(|q| q.quest_id == quest_id) {
                                            messages_to_send.push(ServerMessage::Error { payload: "你已经接取过这个任务了。".to_string() });
                                        } else if session.player.accept_quest(quest) {
                                            messages_to_send.push(ServerMessage::Info { payload: format!("\x1b[1;33m[任务接取] 你接取了任务：{}。输入 'qs' 可查看详细进度。\x1b[0m", quest.name) });
                                        } else {
                                            messages_to_send.push(ServerMessage::Error { payload: "接取任务失败。".to_string() });
                                        }
                                    } else {
                                        messages_to_send.push(ServerMessage::Error { payload: "没有找到这个任务。".to_string() });
                                    }
                                } else {
                                    messages_to_send.push(ServerMessage::Error { payload: "这里没有告示牌，去野外入口找找看吧。".to_string() });
                                }
                            }
                            Command::Get { item } => {
                                let room_items = world.get_items_in_room(&current_room_id_str);
                                let mut found_item_id = None;
                                for id in room_items {
                                    if let Some(proto) = world.static_data.item_prototypes.get(&id) {
                                        if proto.name == item || id.to_string() == item {
                                            found_item_id = Some(id);
                                            break;
                                        }
                                    }
                                }

                                if let Some(item_id) = found_item_id {
                                    if world.remove_item_from_room(&current_room_id_str, item_id) {
                                        session.player.inventory.push(item_id);
                                        let item_name = world.static_data.item_prototypes.get(&item_id)
                                            .map(|p| p.name.clone())
                                            .unwrap_or_else(|| "未知物品".to_string());
                                        messages_to_send.push(ServerMessage::Info { payload: format!("你捡起了{}。", item_name) });
                                    } else {
                                        messages_to_send.push(ServerMessage::Error { payload: "捡起物品失败。".to_string() });
                                    }
                                } else {
                                    messages_to_send.push(ServerMessage::Error { payload: format!("这里没有 {}。", item) });
                                }
                            }
                            Command::Inventory => {
                                if session.player.inventory.is_empty() {
                                    messages_to_send.push(ServerMessage::Info { payload: "你两手空空。".to_string() });
                                } else {
                                    let mut inv_text = String::from("\x1b[1;33m你身上带着：\x1b[0m\n");
                                    for item_id in &session.player.inventory {
                                        let item_name = world.static_data.item_prototypes.get(item_id)
                                            .map(|p| p.name.clone())
                                            .unwrap_or_else(|| "未知物品".to_string());
                                        inv_text.push_str(&format!("- {}\n", item_name));
                                    }
                                    messages_to_send.push(ServerMessage::Description { payload: inv_text });
                                }
                            }
                            Command::Quest => {
                                if session.player.active_quests.is_empty() {
                                    messages_to_send.push(ServerMessage::Info { payload: "当前没有任何进行中的任务。".to_string() });
                                } else {
                                    let mut output = String::from("\x1b[1;33m进行中的任务：\x1b[0m\n");
                                    for status in &session.player.active_quests {
                                        if let Some(quest) = world.static_data.quests.get(&status.quest_id) {
                                            let step_desc = if quest.quest_type == "kill" {
                                                let count = status.kill_counts.get(&quest.target_id).unwrap_or(&0);
                                                format!("{}: {}/{}", quest.description, count, quest.target_count.unwrap_or(0))
                                            } else {
                                                quest.steps.get(status.current_step as usize)
                                                    .map(|s| s.description.as_str())
                                                    .unwrap_or("已完成所有步骤。")
                                                    .to_string()
                                            };
                                            output.push_str(&format!("- {}: {}\n", quest.name, step_desc));
                                        }
                                    }
                                    messages_to_send.push(ServerMessage::Description { payload: output });
                                }
                            }
                            Command::Look => {
                                let desc = get_full_room_description(&current_room_id_str, world);
                                messages_to_send.push(ServerMessage::Description { payload: desc });
                                
                                // Check if looking at a QuestBoard to show available quests
                                let npcs = world.get_npcs_in_room(&current_room_id_str);
                                if npcs.iter().any(|n| n.prototype_id == 2000) {
                                    let mut available = Vec::new();
                                    for quest in world.static_data.quests.values() {
                                        if quest.quest_type == "kill" && 
                                           !session.player.completed_quests.contains(&quest.id) &&
                                           !session.player.active_quests.iter().any(|q| q.quest_id == quest.id) {
                                            available.push(format!("- [{}] {}", quest.id, quest.name));
                                        }
                                    }
                                    if !available.is_empty() {
                                        let mut board_msg = String::from("\n\x1b[1;33m告示牌上贴着以下悬赏：\x1b[0m\n");
                                        board_msg.push_str(&available.join("\n"));
                                        board_msg.push_str("\n\x1b[1;37m输入 'accept <任务ID>' 即可接取。\x1b[0m");
                                        messages_to_send.push(ServerMessage::Info { payload: board_msg });
                                    }
                                }
                            }
                            Command::Score => {
                                let score_str = session.player.get_score_string(&world.static_data.config);
                                messages_to_send.push(ServerMessage::Description { payload: score_str });
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