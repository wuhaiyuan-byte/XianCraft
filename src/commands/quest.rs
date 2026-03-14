use crate::world::world_player::Player;
use crate::world::world_state::WorldState;
use crate::{AppState, ServerMessage};
use colored::*;
use std::sync::Arc;

pub fn handle_quest(player: &Player, world: &WorldState) -> ServerMessage {
    if player.active_quests.is_empty() {
        return ServerMessage::Info {
            payload: "当前没有任何进行中的任务。".to_string(),
        };
    }

    let mut output = format!(
        "{}
",
        "进行中的任务：".yellow().bold()
    );
    for status in &player.active_quests {
        if let Some(quest) = world.static_data.quests.get(&status.quest_id) {
            let step_desc = if quest.quest_type == "kill" {
                let count = status.kill_counts.get(&quest.target_id).unwrap_or(&0);
                format!(
                    "{}: {}/{}",
                    quest.description,
                    count,
                    quest.target_count.unwrap_or(0)
                )
            } else {
                quest
                    .steps
                    .get(status.current_step as usize)
                    .map(|s| s.description.as_str())
                    .unwrap_or("已完成所有步骤。")
                    .to_string()
            };
            output.push_str(&format!("- {}: {}\n", quest.name, step_desc));
        }
    }
    ServerMessage::Description { payload: output }
}

pub fn handle_accept(
    player: &mut Player,
    quest_id: &str,
    current_room_id: &str,
    world: &WorldState,
) -> Option<ServerMessage> {
    let npcs = world.get_npcs_in_room(current_room_id);
    let has_board = npcs.iter().any(|n| n.prototype_id == 2000);

    if !has_board {
        return Some(ServerMessage::Error {
            payload: "这里没有告示牌，去野外入口找找看吧。".to_string(),
        });
    }

    let quest = world.static_data.quests.get(quest_id)?;

    if player.completed_quests.contains(quest_id) {
        return Some(ServerMessage::Error {
            payload: "你已经完成了这个任务，不能重复接取。".to_string(),
        });
    }

    if player.active_quests.iter().any(|q| q.quest_id == quest_id) {
        return Some(ServerMessage::Error {
            payload: "你已经接取过这个任务了。".to_string(),
        });
    }

    if player.accept_quest(quest) {
        Some(ServerMessage::Info {
            payload: format!(
                "{}",
                format!(
                    "[任务接取] 你接取了任务：{}。输入 'qs' 可查看详细进度。",
                    quest.name
                )
                .yellow()
                .bold()
            ),
        })
    } else {
        Some(ServerMessage::Error {
            payload: "接取任务失败。".to_string(),
        })
    }
}
