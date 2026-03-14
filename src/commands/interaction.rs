use crate::world::world_player::{Player, PlayerQuestStatus};
use crate::world::world_state::WorldState;
use crate::{AppState, ServerMessage};
use colored::*;
use std::sync::Arc;

pub fn handle_talk(
    player: &mut Player,
    target: &str,
    current_room_id: &str,
    world: &WorldState,
) -> Option<ServerMessage> {
    let npcs = world.get_npcs_in_room(current_room_id);
    let npc = npcs
        .iter()
        .find(|n| n.name == target || n.prototype_id.to_string() == target)?;

    let mut dialog_id = None;
    let mut quest_finished = false;
    let mut quest_reward = None;
    let mut quest_name = String::new();
    let mut finished_quest_id = String::new();

    for status in &mut player.active_quests {
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
        npc.dialog_id
            .clone()
            .unwrap_or_else(|| "default_greet".to_string())
    });

    let mut payload = format!("{}: {}", npc.name, final_dialog);
    let mut msgs = Vec::new();

    if !quest_name.is_empty() {
        if quest_finished {
            payload.push_str(&format!(
                "
{}",
                format!("[任务完成] {}", quest_name).yellow().bold()
            ));
            if let Some(r) = quest_reward {
                let reward_text = player.grant_reward(&r);
                msgs.push(ServerMessage::Info {
                    payload: reward_text,
                });
            }
            player.active_quests.retain(|q| {
                if let Some(qd) = world.static_data.quests.get(&q.quest_id) {
                    if qd.quest_type == "kill" {
                        if q.is_completed {
                            player.completed_quests.insert(q.quest_id.clone());
                            return false;
                        }
                    } else if q.current_step as usize >= qd.steps.len() {
                        player.completed_quests.insert(q.quest_id.clone());
                        return false;
                    }
                }
                true
            });

            if finished_quest_id == "tutorial_1" && npc.prototype_id == 1002 {
                if world.static_data.quests.contains_key("q102")
                    && !player.completed_quests.contains("q102")
                {
                    player.active_quests.push(PlayerQuestStatus {
                        quest_id: "q102".to_string(),
                        current_step: 0,
                        is_completed: false,
                        kill_counts: std::collections::HashMap::new(),
                    });
                    payload.push_str(&format!(
                        "
{}",
                        "[任务接取] 老村长交给了你一个新的任务：勤能补拙。输入 'qs' 查看详情。"
                            .yellow()
                            .bold()
                    ));
                }
            }
        } else {
            payload.push_str(&format!(
                "
{}",
                format!("[任务更新] {}", quest_name).green().bold()
            ));
        }
    } else {
        if npc.prototype_id == 1002
            && player.completed_quests.contains("tutorial_1")
            && !player.completed_quests.contains("q102")
            && !player.active_quests.iter().any(|q| q.quest_id == "q102")
        {
            if world.static_data.quests.contains_key("q102") {
                player.active_quests.push(PlayerQuestStatus {
                    quest_id: "q102".to_string(),
                    current_step: 0,
                    is_completed: false,
                    kill_counts: std::collections::HashMap::new(),
                });
                payload.push_str(&format!(
                    "
{}",
                    "[任务接取] 老村长交给了你一个新的任务：勤能补拙。输入 'qs' 查看详情。"
                        .yellow()
                        .bold()
                ));
            }
        }
    }

    msgs.insert(0, ServerMessage::Description { payload });
    Some(msgs.pop()?)
}

pub fn handle_get(
    player: &mut Player,
    item: &str,
    current_room_id: &str,
    world: &WorldState,
) -> Option<ServerMessage> {
    let room_items = world.get_items_in_room(current_room_id);
    let mut found_item_id = None;

    for id in room_items {
        if let Some(proto) = world.static_data.item_prototypes.get(&id) {
            if proto.name == item || id.to_string() == item {
                found_item_id = Some(id);
                break;
            }
        }
    }

    let item_id = found_item_id?;

    if world.remove_item_from_room(current_room_id, item_id) {
        player.inventory.push(item_id);
        let item_name = world
            .static_data
            .item_prototypes
            .get(&item_id)
            .map(|p| p.name.clone())
            .unwrap_or_else(|| "未知物品".to_string());
        Some(ServerMessage::Info {
            payload: format!("你捡起了{}。", item_name),
        })
    } else {
        Some(ServerMessage::Error {
            payload: "捡起物品失败。".to_string(),
        })
    }
}
