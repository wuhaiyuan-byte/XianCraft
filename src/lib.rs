
#[cfg(test)]
mod tests;
pub mod command;
pub mod npc;
pub mod world;
pub mod world_model;
pub mod combat;

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
use tokio::time::{sleep, Duration};
use tower_http::services::ServeDir;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Import the colored crate functionality
use colored::*;

use crate::command::{parse, Command};
use crate::npc::Npc;
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

// This function builds the welcome message using truecolor.
fn build_welcome_message() -> String {
    // Define our palette
    let pink = (255, 105, 180); // Hot Pink
    let purple = (218, 112, 214); // Orchid
    let white = (255, 255, 255);

    let line1 = format!("      {}  {}  {}",
        "✧･ﾟ: *✧･ﾟ:*".truecolor(pink.0, pink.1, pink.2),
        "ଘ(◕‿◕✿)ଓ".truecolor(purple.0, purple.1, purple.2),
        "*:･ﾟ✧*:･ﾟ✧".truecolor(pink.0, pink.1, pink.2)
    );

    let line2 = format!("    {}", "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".truecolor(purple.0, purple.1, purple.2));

    let line3 = format!("       {} {}",
        "(つ◕౪◕)つ".truecolor(pink.0, pink.1, pink.2),
        "⚔️  剑 灵 少 女 の 招 待  ⚔️".bold().truecolor(white.0, white.1, white.2)
    );
    
    let line4 = line2.clone();

    let line6 = format!("      {}", "“欧尼酱！快握紧这把灵剑，一起踏上登仙之路吧~”".bold().truecolor(pink.0, pink.1, pink.2));
    
    let line8 = format!("             {}  {}  {}  {}  {}",
        "✦".truecolor(purple.0, purple.1, purple.2),
        "✧".truecolor(pink.0, pink.1, pink.2),
        "(ﾉ◕ヮ◕)ﾉ*:･ﾟ✧".truecolor(white.0, white.1, white.2),
        "✧".truecolor(pink.0, pink.1, pink.2),
        "✦".truecolor(purple.0, purple.1, purple.2)
    );

    format!("{}
{}
{}
{}
{}

{}

{}
", line1, line2, line3, line4, line6, line8, "")
}

pub(crate) fn realm_level_to_name(level: u16, sub_level: u16) -> String {
    match level {
        1 => format!("炼气{}层", match sub_level {
            1 => "一", 2 => "二", 3 => "三", 4 => "四", 5 => "五",
            6 => "六", 7 => "七", 8 => "八", 9 => "九", _ => "?",
        }),
        2 => format!("筑基{}层", match sub_level {
            1 => "一", 2 => "二", 3 => "三", 4 => "四", 5 => "五",
            6 => "六", 7 => "七", 8 => "八", 9 => "九", _ => "?",
        }),
        3 => format!("金丹{}层", match sub_level {
            1 => "一", 2 => "二", 3 => "三", 4 => "四", 5 => "五",
            6 => "六", 7 => "七", 8 => "八", 9 => "九", _ => "?",
        }),
        4 => format!("元婴{}层", match sub_level {
            1 => "一", 2 => "二", 3 => "三", 4 => "四", 5 => "五",
            6 => "六", 7 => "七", 8 => "八", 9 => "九", _ => "?",
        }),
        5 => format!("化神{}层", match sub_level {
            1 => "一", 2 => "二", 3 => "三", 4 => "四", 5 => "五",
            6 => "六", 7 => "七", 8 => "八", 9 => "九", _ => "?",
        }),
        _ => format!("境界{}", level),
    }
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

pub(crate) fn generate_who_list(state: &Arc<AppState>, use_color: bool) -> String {
    let players_data: Vec<(String, u16, u16, String, u64, bool)> = {
        let sessions = state.player_sessions.lock().unwrap();
        sessions
            .values()
            .filter(|s| s.user_id.is_some())
            .map(|s| {
                (
                    s.player.name.clone(),
                    s.player.realm_level,
                    s.player.realm_sub_level,
                    s.player.sect.clone().unwrap_or_else(|| "无".to_string()),
                    s.player.id,
                    s.player.is_resting,
                )
            })
            .collect()
    };
    
    let world = &state.world_state;
    
    let mut players: Vec<(String, String, String, String)> = players_data
        .into_iter()
        .map(|(name, realm_level, realm_sub_level, sect, player_id, is_resting)| {
            let realm = realm_level_to_name(realm_level, realm_sub_level);
            let status = {
                let npcs = world.get_npcs_in_room(&world.get_player_room_id(player_id).unwrap_or_default());
                let in_combat = npcs.iter().any(|npc| {
                    if let Some(proto) = world.static_data.npc_prototypes.get(&npc.prototype_id) {
                        (proto.ai == "monster" || !proto.flags.contains(&"friendly".to_string())) 
                            && npc.combat_target == Some(player_id)
                    } else {
                        false
                    }
                });
                if in_combat {
                    "战斗中".to_string()
                } else if is_resting {
                    "打坐中".to_string()
                } else {
                    "游历中".to_string()
                }
            };
            (name, realm, sect, status)
        })
        .collect();
    
    players.sort_by(|a, b| {
        let a_level = parse_realm_level(&a.1);
        let b_level = parse_realm_level(&b.1);
        b_level.cmp(&a_level)
    });
    
    let count = players.len();
    
    let name_width = 16;
    let realm_width = 16;
    let sect_width = 12;
    let status_width = 12;
    
    let header_line = format!("┌{}┬{}┬{}┬{}┐", 
        "─".repeat(name_width), "─".repeat(realm_width), "─".repeat(sect_width), "─".repeat(status_width));
    let sep_line = format!("├{}┼{}┼{}┼{}┤", 
        "─".repeat(name_width), "─".repeat(realm_width), "─".repeat(sect_width), "─".repeat(status_width));
    let footer_line = format!("└{}┴{}┴{}┴{}┘", 
        "─".repeat(name_width), "─".repeat(realm_width), "─".repeat(sect_width), "─".repeat(status_width));
    
    let mut output = String::new();
    
    if use_color {
        output.push_str(&format!("{}\n", "【 仙 界 同 道 】".magenta().bold()));
        output.push_str(&format!("{}\n", header_line.truecolor(180, 100, 200)));
        output.push_str(&format!("│{}│{}│{}│{}│\n", 
            pad_to_width("姓 名", name_width - 2).white(),
            pad_to_width("境 界", realm_width - 2).white(),
            pad_to_width("宗 门", sect_width - 2).white(),
            pad_to_width("当前状态", status_width - 2).white()));
        output.push_str(&format!("{}\n", sep_line.truecolor(180, 100, 200)));
    } else {
        output.push_str("【 仙 界 同 道 】\n");
        output.push_str(&format!("{}\n", header_line));
        output.push_str(&format!("│{}│{}│{}│{}│\n", 
            pad_to_width("姓 名", name_width - 2),
            pad_to_width("境 界", realm_width - 2),
            pad_to_width("宗 门", sect_width - 2),
            pad_to_width("当前状态", status_width - 2)));
        output.push_str(&format!("{}\n", sep_line));
    }
    
    if players.is_empty() {
        let empty_msg = "暂无其他玩家在线";
        if use_color {
            output.push_str(&format!("│{}│{}│{}│{}│\n", 
                " ".repeat(name_width - 2), " ".repeat(realm_width - 2), 
                pad_to_width(empty_msg, sect_width - 2).yellow(), " ".repeat(status_width - 2)));
        } else {
            output.push_str(&format!("│{}│{}│{}│{}│\n", 
                " ".repeat(name_width - 2), " ".repeat(realm_width - 2), 
                pad_to_width(empty_msg, sect_width - 2), " ".repeat(status_width - 2)));
        }
    } else {
        for (name, realm, sect, status) in &players {
            if use_color {
                let status_str = if status == "战斗中" {
                    format!("{}", status.red().bold())
                } else if status == "打坐中" {
                    format!("{}", status.cyan())
                } else {
                    format!("{}", status.green())
                };
                output.push_str(&format!("│{}│{}│{}│{}│\n", 
                    pad_to_width(name, name_width - 2).green().bold(), 
                    pad_to_width(realm, realm_width - 2).truecolor(200, 150, 255), 
                    pad_to_width(sect, sect_width - 2).yellow(),
                    pad_to_width(&status_str, status_width - 2)));
            } else {
                output.push_str(&format!("│{}│{}│{}│{}│\n", 
                    pad_to_width(name, name_width - 2), 
                    pad_to_width(realm, realm_width - 2), 
                    pad_to_width(sect, sect_width - 2),
                    pad_to_width(status, status_width - 2)));
            }
        }
    }
    
    if use_color {
        output.push_str(&format!("{}\n", footer_line.truecolor(180, 100, 200)));
        output.push_str(&format!("{}", format!("  ★ 当前共有 {}位 道友在线 ★ ", count).white().bold()));
    } else {
        output.push_str(&format!("{}\n", footer_line));
        output.push_str(&format!("{}", format!("  ★ 当前共有 {}位 道友在线 ★ ", count)));
    }
    
    output
}

fn pad_to_width(s: &str, width: usize) -> String {
    let char_count = s.chars().count();
    if char_count >= width {
        return s.to_string();
    }
    let padding = width - char_count;
    let left = padding / 2;
    let right = padding - left;
    format!("{}{}{}", " ".repeat(left), s, " ".repeat(right))
}

fn parse_realm_level(realm: &str) -> (u16, u16) {
    let level = if realm.starts_with("炼气") {
        1
    } else if realm.starts_with("筑基") {
        2
    } else if realm.starts_with("金丹") {
        3
    } else if realm.starts_with("元婴") {
        4
    } else if realm.starts_with("化神") {
        5
    } else {
        0
    };
    
    let sub = if realm.contains("一") {
        1
    } else if realm.contains("二") {
        2
    } else if realm.contains("三") {
        3
    } else if realm.contains("四") {
        4
    } else if realm.contains("五") {
        5
    } else if realm.contains("六") {
        6
    } else if realm.contains("七") {
        7
    } else if realm.contains("八") {
        8
    } else if realm.contains("九") {
        9
    } else {
        0
    };
    
    (level, sub)
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

    // Use STATIC_DIR env var for the frontend assets, defaulting to "client/dist" for local dev.
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
                        payload: "[提示]：你现在应该尝试输入 work 指令来进行伐木。记得随时输入 score 查看你的体力值。".cyan().to_string() 
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

fn get_full_room_description(
    room_id: &str, 
    world_state: &WorldState, 
    other_players: Vec<String>,
    npcs_in_room: Vec<Npc>,
    room_items: Vec<u32>,
) -> String {
    if let Some(room) = world_state.get_room(room_id) {
        let mut full_desc = format!("{}
{}", room.name.cyan().bold(), room.description);

        if !other_players.is_empty() {
            let player_list: Vec<String> = other_players.iter().map(|n| n.green().to_string()).collect();
            full_desc.push_str(&format!("
你在此处看到了：{}", player_list.join(", ")));
        }

        if !npcs_in_room.is_empty() {
            let npc_names: Vec<String> = npcs_in_room
                .iter()
                .map(|npc| npc.name.green().to_string())
                .collect();
            full_desc.push_str(&format!("
● {}", npc_names.join(", ")));
        }

        if !room_items.is_empty() {
            let item_names: Vec<String> = room_items
                .iter()
                .filter_map(|id| world_state.static_data.item_prototypes.get(id))
                .map(|item| item.name.clone())
                .collect();
            if !item_names.is_empty() {
                full_desc.push_str(&format!("
{}", item_names.join(", ")));
            }
        }

        if !room.exits.is_empty() {
            let exit_keys: Vec<String> = room.exits.keys().cloned().collect();
            full_desc.push_str(&format!("
{}", format!("出口: [{}]", exit_keys.join(", ")).white()));
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
                                // *** THIS IS THE CHANGED LINE ***
                                let mut welcome_content = format!("{}

{}", build_welcome_message(), get_full_room_description("genesis_altar", world, Vec::new(), Vec::new(), Vec::new()));
                                
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
    let mut messages_to_send = Vec::new();

    if matches!(command, Command::Who) {
        tracing::info!("[WHO] Player {} requested who list", player_id);
        let who_output = generate_who_list(&state, true);
        messages_to_send.push(ServerMessage::Description { payload: who_output });
    } else {
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
                        Command::Look | Command::Score | Command::Quest | Command::Inventory | Command::Rest | Command::Help => false,
                        _ => session.player.is_resting,
                    };

                    if block_resting {
                        messages_to_send.push(ServerMessage::Error { payload: "你正在休息中，无法执行此操作。".to_string() });
                    } else {
                        match command {
                            Command::Help => {
                                let help_text = format!("{}
{}
{}
{}
{}
{}
{}
{}
{}
{}
{}
{}
{}
{}
{}
{}
{}
{}
{}
{}",
                                    "----【 可用指令 (Commands) 】----".bold().yellow(),
                                    "
  ".to_string() + &"【通用】".bold().cyan(),
                                    "  look              - 查看当前环境。",
                                    "  score/status      - 查看你的角色状态。",
                                    "  inventory/i       - 查看你的背包。",
                                    "  say <内容>        - 对房间里的所有人说话。",
                                    "
  ".to_string() + &"【移动】".bold().cyan(),
                                    "  go <方向>         - 向指定方向移动 (north, south, east, west... 或 n, s, e, w...)",
                                    "
  ".to_string() + &"【互动】".bold().cyan(),
                                    "  talk <目标>       - 与NPC对话。",
                                    "  attack <目标>     - 攻击一个目标。",
                                    "  get/take <物品>   - 从地上捡起物品。",
                                    "
  ".to_string() + &"【任务】".bold().cyan(),
                                    "  quest/qs          - 查看当前任务状态。",
                                    "  accept <任务ID>   - 从告示牌等处接受任务。",
                                    "
  ".to_string() + &"【其它】".bold().cyan(),
                                    "  rest              - 原地休息以恢复体力。",
                                    "  work              - 在特定地点劳动以赚取奖励。",
                                    "  who               - 查看当前在线的玩家。",
                                    ""
                                );
                                messages_to_send.push(ServerMessage::Info { payload: help_text });
                            }
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
                            Command::Kill { target } => {
                                let attacker_stats = {
                                    let p = &session.player;
                                    combat::CombatStats {
                                        hp: p.hp as i32,
                                        max_hp: p.hp_max as i32,
                                        attack: p.atk as i32,
                                        defense: 5,
                                        level: p.realm_level as i32,
                                        name: p.name.clone(),
                                        is_player: true,
                                        str: p.stats.str as i32,
                                        dex: p.stats.dex as i32,
                                        int: p.stats.int as i32,
                                    }
                                };
                                let target_npc = {
                                    let data = world.dynamic_data.lock().unwrap();
                                    data.npcs.values()
                                        .find(|n| n.current_room == current_room_id_str && (n.name == target || n.prototype_id.to_string() == target))
                                        .cloned()
                                };
                                
                                if let Some(npc) = target_npc {
                                    let defender_stats = combat::CombatStats {
                                        hp: npc.hp,
                                        max_hp: npc.max_hp,
                                        attack: npc.attack,
                                        defense: npc.defense,
                                        level: npc.level,
                                        name: npc.name.clone(),
                                        is_player: false,
                                        str: 10,
                                        dex: 10,
                                        int: 10,
                                    };
                                    
                                    let result = combat::resolve_attack(&attacker_stats, &defender_stats, None);
                                    
                                    match result {
                                        combat::CombatResult::Hit { damage, is_crit: _, log } => {
                                            messages_to_send.push(ServerMessage::Description { payload: log });
                                            
                                            let mut dynamic_data = world.dynamic_data.lock().unwrap();
                                            if let Some(npc_instance) = dynamic_data.npcs.get_mut(&npc.instance_id) {
                                                npc_instance.hp -= damage;
                                                if npc_instance.hp <= 0 {
                                                    dynamic_data.npcs.remove(&npc.instance_id);
                                                    messages_to_send.push(ServerMessage::Description { payload: format!("你击败了{}！", npc.name.yellow()) });
                                                }
                                            }
                                        }
                                        combat::CombatResult::TargetKilled { damage: _, is_crit: _, log } => {
                                            messages_to_send.push(ServerMessage::Description { payload: log });
                                            
                                            let mut dynamic_data = world.dynamic_data.lock().unwrap();
                                            dynamic_data.npcs.remove(&npc.instance_id);
                                            messages_to_send.push(ServerMessage::Description { payload: format!("你击败了{}！", npc.name.yellow()) });
                                        }
                                        combat::CombatResult::Miss { log } => {
                                            messages_to_send.push(ServerMessage::Description { payload: log });
                                        }
                                        _ => {}
                                    }
                                } else {
                                    messages_to_send.push(ServerMessage::Error { payload: format!("这里没有 {}。", target) });
                                }
                            }
                            Command::Cast { skill, target } => {
                                let skill_template = world.static_data.skills.get(&skill).cloned();
                                if let Some(skill_tpl) = skill_template {
                                    if session.player.qi < skill_tpl.cost_qi as u32 {
                                        messages_to_send.push(ServerMessage::Error { payload: format!("你的真元不足，需要 {} 点真元。", skill_tpl.cost_qi) });
                                    } else {
                                        let attacker_stats = {
                                            let p = &session.player;
                                            combat::CombatStats {
                                                hp: p.hp as i32,
                                                max_hp: p.hp_max as i32,
                                                attack: p.atk as i32,
                                                defense: 5,
                                                level: p.realm_level as i32,
                                                name: p.name.clone(),
                                                is_player: true,
                                                str: p.stats.str as i32,
                                                dex: p.stats.dex as i32,
                                                int: p.stats.int as i32,
                                            }
                                        };
                                        
                                        if skill_tpl.is_magic && (skill_tpl.base_damage as i32) < 0 {
                                            let result = combat::resolve_heal(&skill_tpl, &attacker_stats);
                                            if let combat::CombatResult::Heal { amount, log } = result {
                                                session.player.qi -= skill_tpl.cost_qi as u32;
                                                session.player.hp = (session.player.hp + amount as u32).min(session.player.hp_max);
                                                messages_to_send.push(ServerMessage::Description { payload: log });
                                            }
                                        } else {
                                            let target_npc = if let Some(t) = target {
                                                let data = world.dynamic_data.lock().unwrap();
                                                data.npcs.values()
                                                    .find(|n| n.current_room == current_room_id_str && (n.name == t || n.prototype_id.to_string() == t))
                                                    .cloned()
                                            } else {
                                                None
                                            };
                                            
                                            if let Some(npc) = target_npc {
                                                let defender_stats = combat::CombatStats {
                                                    hp: npc.hp,
                                                    max_hp: npc.max_hp,
                                                    attack: npc.attack,
                                                    defense: npc.defense,
                                                    level: npc.level,
                                                    name: npc.name.clone(),
                                                    is_player: false,
                                                    str: 10,
                                                    dex: 10,
                                                    int: 10,
                                                };
                                                
                                                let result = combat::resolve_attack(&attacker_stats, &defender_stats, Some(&skill_tpl));
                                                
                                                session.player.qi -= skill_tpl.cost_qi as u32;
                                                
                                                match result {
                                                    combat::CombatResult::Hit { damage, is_crit: _, log } => {
                                                        messages_to_send.push(ServerMessage::Description { payload: log });
                                                        let mut dynamic_data = world.dynamic_data.lock().unwrap();
                                                        if let Some(npc_instance) = dynamic_data.npcs.get_mut(&npc.instance_id) {
                                                            npc_instance.hp -= damage;
                                                            if npc_instance.hp <= 0 {
                                                                dynamic_data.npcs.remove(&npc.instance_id);
                                                                messages_to_send.push(ServerMessage::Description { payload: format!("你击败了{}！", npc.name.yellow()) });
                                                            }
                                                        }
                                                    }
                                                    combat::CombatResult::TargetKilled { damage: _, is_crit: _, log } => {
                                                        messages_to_send.push(ServerMessage::Description { payload: log });
                                                        let mut dynamic_data = world.dynamic_data.lock().unwrap();
                                                        dynamic_data.npcs.remove(&npc.instance_id);
                                                        messages_to_send.push(ServerMessage::Description { payload: format!("你击败了{}！", npc.name.yellow()) });
                                                    }
                                                    combat::CombatResult::Miss { log } => {
                                                        messages_to_send.push(ServerMessage::Description { payload: log });
                                                    }
                                                    _ => {}
                                                }
                                            } else {
                                                messages_to_send.push(ServerMessage::Error { payload: "你的目标不存在。".to_string() });
                                            }
                                        }
                                    }
                                } else {
                                    messages_to_send.push(ServerMessage::Error { payload: format!("没有找到技能: {}", skill) });
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
                                    messages_to_send.push(ServerMessage::Info { payload: format!("{}", "获得奖励：灵贝+20，修为+5，潜能+2".green().bold()) });

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
                                        messages_to_send.push(ServerMessage::Description { payload: format!("{}", "【机缘】随着最后一斧劈下，你感到一股清凉的气流顺着指尖流向全身。你对天地的感悟达到了新的高度！请回广场向村长报告。".magenta().bold()) });
                                    }
                                }
                            }
                            Command::Go { direction } => {
                                    if let Some(next_room_id) = current_room.exits.get(&direction) {
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

                                            // Check for next quest: q102 after tutorial_1 if talking to chief (1002)
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
                                let npcs = world.get_npcs_in_room(&current_room_id_str);
                                let has_board = npcs.iter().any(|n| n.prototype_id == 2000);
                                
                                if has_board {
                                    if let Some(quest) = world.static_data.quests.get(&quest_id) {
                                        if session.player.completed_quests.contains(&quest_id) {
                                            messages_to_send.push(ServerMessage::Error { payload: "你已经完成了这个任务，不能重复接取。".to_string() });
                                        } else if session.player.active_quests.iter().any(|q| q.quest_id == quest_id) {
                                            messages_to_send.push(ServerMessage::Error { payload: "你已经接取过这个任务了。".to_string() });
                                        } else if session.player.accept_quest(quest) {
                                            messages_to_send.push(ServerMessage::Info { payload: format!("{}", format!("[任务接取] 你接取了任务：{}。输入 'qs' 可查看详细进度。", quest.name).yellow().bold()) });
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
                                    let mut inv_text = format!("{}
", "你身上带着：".yellow().bold());
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
                                    let mut output = format!("{}
", "进行中的任务：".yellow().bold());
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
                                tracing::info!("[LOOK] Player {} looking at room {}", player_id, current_room_id_str);
                                let (other_players, npc_data, room_items_data) = {
                                    let data = world.dynamic_data.lock().unwrap();
                                    let players: Vec<String> = data.players.iter()
                                        .filter(|(id, loc)| **id != player_id && loc.room_id == current_room_id_str)
                                        .filter_map(|(_, loc)| loc.user_name.clone())
                                        .collect();
                                    let npcs: Vec<Npc> = data.npcs.values()
                                        .filter(|npc| npc.current_room == current_room_id_str)
                                        .cloned()
                                        .collect();
                                    let items: Vec<u32> = data.room_items.get(&current_room_id_str).cloned().unwrap_or_default();
                                    (players, npcs, items)
                                };
                                let desc = get_full_room_description(&current_room_id_str, world, other_players, npc_data.clone(), room_items_data);
                                messages_to_send.push(ServerMessage::Description { payload: desc });
                                
                                if npc_data.iter().any(|n| n.prototype_id == 2000) {
                                    let mut available = Vec::new();
                                    for quest in world.static_data.quests.values() {
                                        if quest.quest_type == "kill" && 
                                           !session.player.completed_quests.contains(&quest.id) &&
                                           !session.player.active_quests.iter().any(|q| q.quest_id == quest.id) {
                                            available.push(format!("- [{}] {}", quest.id, quest.name));
                                        }
                                    }
                                    if !available.is_empty() {
                                        let mut board_msg = format!("{}
 ", "
 告示牌上贴着以下悬赏：".yellow().bold());
                                        board_msg.push_str(&available.join("
"));
                                        board_msg.push_str(&format!("
{}", "
 输入 'accept <任务ID>' 即可接取.".white().bold()));
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
    }

    for server_msg in messages_to_send {
        if let Ok(json_str) = serde_json::to_string(&server_msg) {
            let _ = sender.send(Message::Text(json_str)).await;
        }
    }
}
