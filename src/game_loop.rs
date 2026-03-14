use std::sync::Arc;
use std::time::SystemTime as StdSystemTime;
use tokio::time::{sleep, Duration};
use crate::{AppState, ServerMessage, combat};
use colored::*;
use axum::extract::ws::Message;

pub async fn game_loop(app_state: Arc<AppState>) {
    let mut tick_counter: u64 = 0;
    const RECOVERY_TICK_INTERVAL: u64 = 10;
    
    let combat_tick_ms = app_state.world_state.static_data.config.combat_tick_ms;
    let combat_tick_interval = combat_tick_ms / 1000;
    if combat_tick_interval == 0 {
        tracing::warn!("combat_tick_ms too small, using default 1 second");
    }

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

        if tick_counter % combat_tick_interval.max(1) == 0 {
            process_combat_ticks(&app_state).await;
        }
    }
}

async fn process_combat_ticks(app_state: &Arc<AppState>) {
    #[derive(Clone)]
    struct CombatEntity {
        id: u64,
        is_player: bool,
        hp: i32,
        max_hp: i32,
        atk: i32,
        defense: i32,
        level: i32,
        name: String,
        target_id: Option<String>,
        target_is_player: bool,
        skill_id: String,
        combo_index: usize,
    }
    
    let mut combat_entities: Vec<CombatEntity> = Vec::new();
    
    {
        let sessions = app_state.player_sessions.lock().unwrap();
        for (id, session) in sessions.iter() {
            if session.user_id.is_some() {
                if let Some(cs) = &session.player.combat_state {
                    combat_entities.push(CombatEntity {
                        id: *id,
                        is_player: true,
                        hp: session.player.hp as i32,
                        max_hp: session.player.hp_max as i32,
                        atk: session.player.atk as i32,
                        defense: 5,
                        level: session.player.realm_level as i32,
                        name: session.player.name.clone(),
                        target_id: Some(cs.target_id.clone()),
                        target_is_player: cs.target_is_player,
                        skill_id: cs.current_skill_id.clone(),
                        combo_index: cs.combo_index,
                    });
                }
            }
        }
    }
    
    {
        let data = app_state.world_state.dynamic_data.lock().unwrap();
        let npcs_in_combat: Vec<_> = data.npcs.iter()
            .filter(|(_, npc)| npc.combat_state.is_some())
            .collect();
        tracing::debug!("NPCs in combat: {:?}", npcs_in_combat.len());
        
        for (instance_id, npc) in npcs_in_combat {
            if let Some(cs) = &npc.combat_state {
                combat_entities.push(CombatEntity {
                    id: *instance_id,
                    is_player: false,
                    hp: npc.hp,
                    max_hp: npc.max_hp as i32,
                    atk: npc.attack as i32,
                    defense: npc.defense as i32,
                    level: npc.level as i32,
                    name: npc.name.clone(),
                    target_id: Some(cs.target_id.clone()),
                    target_is_player: cs.target_is_player,
                    skill_id: cs.current_skill_id.clone(),
                    combo_index: cs.combo_index,
                });
            }
        }
    }
    
    tracing::debug!("Total combat entities: {:?}", combat_entities.len());
    
    let mut updates: Vec<(u64, bool, u64, bool, i32, String, String, bool)> = Vec::new();
    
    for entity in &combat_entities {
        if let Some(target_id_str) = &entity.target_id {
            let target_id: u64 = target_id_str.parse().unwrap_or(0);
            
            let target = combat_entities.iter().find(|e| {
                if entity.target_is_player {
                    e.is_player && e.id == target_id
                } else {
                    !e.is_player && e.id == target_id
                }
            });
            
            if let Some(target) = target {
                let attacker_stats = combat::CombatStats {
                    hp: entity.hp,
                    max_hp: entity.max_hp,
                    attack: entity.atk,
                    defense: entity.defense,
                    level: entity.level,
                    name: entity.name.clone(),
                    is_player: entity.is_player,
                    str: 10,
                    dex: 10,
                    int: 10,
                };
                
                let defender_stats = combat::CombatStats {
                    hp: target.hp,
                    max_hp: target.max_hp,
                    attack: target.atk,
                    defense: target.defense,
                    level: target.level,
                    name: target.name.clone(),
                    is_player: target.is_player,
                    str: 10,
                    dex: 10,
                    int: 10,
                };
                
                let skill = app_state.world_state.static_data.skills.get(&entity.skill_id);
                let skill_name = skill.map(|s| s.name.clone()).unwrap_or_else(|| entity.skill_id.clone());
                let result = combat::resolve_attack(&attacker_stats, &defender_stats, skill);
                
                match result {
                    combat::CombatResult::Hit { damage, log: _, is_crit: _ } => {
                        let new_hp = (target.hp - damage).max(0);
                        let hp_percent = (new_hp as f32 / target.max_hp as f32 * 100.0) as i32;
                        
                        let health_state = if hp_percent > 80 {
                            "真元充沛，毫发无损"
                        } else if hp_percent > 50 {
                            "气息微乱，护体灵光闪烁"
                        } else if hp_percent > 20 {
                            "发丝凌乱，嘴角溢出一丝鲜血"
                        } else {
                            "摇摇欲坠，犹如风中残烛"
                        };
                        
                        let new_hp_defender = new_hp;
                        
                        let msg_to_attacker = if entity.is_player {
                            format!("你对{}使出{},\n造成了{}点伤害。\n{}", 
                                target.name, skill_name, damage, health_state.yellow())
                        } else {
                            format!("{}对你使出{},\n造成了{}点伤害。\n{}", 
                                entity.name, skill_name, damage, health_state.yellow())
                        };
                        
                        let msg_to_defender = if target.is_player {
                            format!("你对{}使出{},\n造成了{}点伤害。\n{}", 
                                entity.name, skill_name, damage, health_state.yellow())
                        } else {
                            format!("{}对你使出{},\n造成了{}点伤害。\n{}", 
                                entity.name, skill_name, damage, health_state.yellow())
                        };
                        
                        updates.push((entity.id, entity.is_player, target.id, target.is_player, new_hp_defender, msg_to_attacker, msg_to_defender, new_hp <= 0));
                    }
                    combat::CombatResult::Miss { log: _ } => {
                        let msg_to_attacker = if entity.is_player {
                            format!("你对{}的攻击落空了！", target.name)
                        } else {
                            format!("{}对我的攻击落空了！", entity.name)
                        };
                        
                        let msg_to_defender = if target.is_player {
                            format!("你对{}的攻击落空了！", entity.name)
                        } else {
                            format!("{}的攻击落空了！", entity.name)
                        };
                        
                        updates.push((entity.id, entity.is_player, target.id, target.is_player, target.hp, msg_to_attacker, msg_to_defender, false));
                    }
                    _ => {}
                }
            }
        }
    }
    
    for (attacker_id, attacker_is_player, defender_id, defender_is_player, new_hp, msg_to_attacker, msg_to_defender, is_dead) in &updates {
        // 玩家攻击 - 发送消息给攻击者
        if *attacker_is_player {
            let mut sessions = app_state.player_sessions.lock().unwrap();
            if let Some(session) = sessions.get_mut(attacker_id) {
                if !msg_to_attacker.is_empty() {
                    let msg = ServerMessage::Description { payload: msg_to_attacker.clone() };
                    if let Ok(json) = serde_json::to_string(&msg) {
                        let _ = session.sender.try_send(Message::Text(json));
                    }
                }
            }
            
            // 如果目标是玩家，更新玩家HP并发送消息
            if *defender_is_player {
                if let Some(session) = sessions.get_mut(defender_id) {
                    session.player.hp = *new_hp as u32;
                    if !msg_to_defender.is_empty() {
                        let msg = ServerMessage::Description { payload: msg_to_defender.clone() };
                        if let Ok(json) = serde_json::to_string(&msg) {
                            let _ = session.sender.try_send(Message::Text(json));
                        }
                    }
                    if *is_dead {
                        session.player.combat_state = None;
                        let death_msg = format!("{}发出了一声不甘的惨叫声，身死道消，化作点点灵光消散于天地间。", session.player.name.yellow());
                        let msg = ServerMessage::Description { payload: death_msg };
                        if let Ok(json) = serde_json::to_string(&msg) {
                            let _ = session.sender.try_send(Message::Text(json));
                        }
                    }
                }
            }
            drop(sessions);
        }
        
        // NPC攻击玩家 - 更新玩家HP并发送被攻击消息
        if !*attacker_is_player && *defender_is_player {
            let mut sessions = app_state.player_sessions.lock().unwrap();
            if let Some(session) = sessions.get_mut(defender_id) {
                session.player.hp = *new_hp as u32;
                if !msg_to_defender.is_empty() {
                    let msg = ServerMessage::Description { payload: msg_to_defender.clone() };
                    if let Ok(json) = serde_json::to_string(&msg) {
                        let _ = session.sender.try_send(Message::Text(json));
                    }
                }
                if *is_dead {
                    session.player.combat_state = None;
                    let death_msg = format!("{}发出了一声不甘的惨叫声，身死道消，化作点点灵光消散于天地间。", session.player.name.yellow());
                    let msg = ServerMessage::Description { payload: death_msg };
                    if let Ok(json) = serde_json::to_string(&msg) {
                        let _ = session.sender.try_send(Message::Text(json));
                    }
                }
            }
        }
    }
    
    for (attacker_id, attacker_is_player, defender_id, defender_is_player, new_hp, _msg_to_attacker, _msg_to_defender, is_dead) in &updates {
        // 只有当NPC死亡时才清除玩家的战斗状态
        if !*defender_is_player && *is_dead {
            let attacker_id: u64 = attacker_id.to_string().parse().unwrap_or(0);
            
            // 移除NPC
            let mut data = app_state.world_state.dynamic_data.lock().unwrap();
            if let Some(npc) = data.npcs.get(defender_id) {
                let npc_proto_id = npc.prototype_id;
                let npc_name = npc.name.clone();
                data.npcs.remove(defender_id);
                drop(data);
                
                // 清除玩家战斗状态并发送消息
                let mut sessions = app_state.player_sessions.lock().unwrap();
                if let Some(session) = sessions.get_mut(&attacker_id) {
                    session.player.combat_state = None;
                    let death_msg = format!("{}发出了一声不甘的惨叫声，身死道消，化作点点灵光消散于天地间。", npc_name.yellow());
                    let msg = ServerMessage::Description { payload: death_msg };
                    if let Ok(json) = serde_json::to_string(&msg) {
                        let _ = session.sender.try_send(Message::Text(json));
                    }
                    
                    let quest_msg = session.player.on_kill(&npc_proto_id.to_string(), &app_state.world_state.static_data.quests);
                    if !quest_msg.is_empty() {
                        if let Ok(json) = serde_json::to_string(&ServerMessage::Info { payload: quest_msg }) {
                            let _ = session.sender.try_send(Message::Text(json));
                        }
                    }
                }
            }
        }
    }
}
