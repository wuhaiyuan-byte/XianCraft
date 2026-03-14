use crate::world::world_player::Player;
use crate::world::world_state::WorldState;
use crate::{AppState, ServerMessage};
use colored::*;
use std::sync::Arc;

pub fn handle_go(
    player: &mut Player,
    direction: &str,
    current_room_id: &str,
    current_room: &crate::world_model::Room,
    world: &WorldState,
    state: &Arc<AppState>,
    player_id: u64,
    user_id: Option<String>,
) -> Option<(ServerMessage, String, String)> {
    if let Some(cs) = &player.combat_state {
        return Some((
            ServerMessage::Error {
                payload: format!("你正在和{}战斗，不能移动！", cs.target_name),
            },
            String::new(),
            String::new(),
        ));
    }

    let next_room_id = current_room.exits.get(direction)?;

    if !player.consume_stamina(1) {
        return Some((
            ServerMessage::Error {
                payload: "你太累了，走不动了。".to_string(),
            },
            String::new(),
            String::new(),
        ));
    }

    let next_room_id_str = next_room_id.clone();
    let user_name = user_id;
    let from_room_id = current_room_id.to_string();

    world.move_player_to_room(player_id, &next_room_id_str, user_name.clone());

    let mut quest_updates = Vec::new();
    for status in &mut player.active_quests {
        if let Some(quest) = world.static_data.quests.get(&status.quest_id) {
            if let Some(step) = quest.steps.get(status.current_step as usize) {
                if step.step_type == "move" && step.target_id == next_room_id_str {
                    status.current_step += 1;
                    quest_updates.push((
                        quest.name.clone(),
                        status.current_step as usize == quest.steps.len(),
                        quest.rewards.clone(),
                    ));
                }
            }
        }
    }

    let mut payload = crate::ui::get_full_room_description(
        &next_room_id_str,
        world,
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    let mut reward_msgs = Vec::new();

    for (name, is_finished, rewards) in quest_updates {
        if is_finished {
            payload.push_str(&format!(
                "
{}",
                format!("[任务完成] {}", name).yellow().bold()
            ));
            let reward_text = player.grant_reward(&rewards);
            reward_msgs.push(ServerMessage::Info {
                payload: reward_text,
            });
        } else {
            payload.push_str(&format!(
                "
{}",
                format!("[任务更新] {}", name).green().bold()
            ));
        }
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

    Some((
        ServerMessage::Description { payload },
        from_room_id,
        next_room_id_str,
    ))
}
