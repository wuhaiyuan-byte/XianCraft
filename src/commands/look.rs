use crate::npc::Npc;
use crate::ui::get_full_room_description;
use crate::world::world_player::Player;
use crate::world::world_state::WorldState;
use crate::ServerMessage;
use colored::*;

pub fn handle_look(
    player_id: u64,
    current_room_id: &str,
    world: &WorldState,
    player: &Player,
) -> ServerMessage {
    let (other_players, npc_data, room_items_data) = {
        let data = world.dynamic_data.lock().unwrap();
        let players: Vec<String> = data
            .players
            .iter()
            .filter(|(id, loc)| **id != player_id && loc.room_id == current_room_id)
            .filter_map(|(_, loc)| loc.user_name.clone())
            .collect();
        let npcs: Vec<Npc> = data
            .npcs
            .values()
            .filter(|npc| npc.current_room == current_room_id)
            .cloned()
            .collect();
        let items: Vec<u32> = data
            .room_items
            .get(&current_room_id.to_string())
            .cloned()
            .unwrap_or_default();
        (players, npcs, items)
    };

    let desc = get_full_room_description(
        current_room_id,
        world,
        other_players,
        npc_data.clone(),
        room_items_data,
    );
    let msg = ServerMessage::Description { payload: desc };

    if npc_data.iter().any(|n| n.prototype_id == 2000) {
        let mut available = Vec::new();
        for quest in world.static_data.quests.values() {
            if quest.quest_type == "kill"
                && !player.completed_quests.contains(&quest.id)
                && !player.active_quests.iter().any(|q| q.quest_id == quest.id)
            {
                available.push(format!("- [{}] {}", quest.id, quest.name));
            }
        }

        if !available.is_empty() {
            let board_msg = format!(
                "{}
  ",
                "
  告示牌上贴着以下悬赏："
                    .yellow()
                    .bold()
            );
            let board_msg = format!(
                "{}{}",
                board_msg,
                available.join(
                    "
 "
                )
            );
            let board_msg = format!(
                "{}{}",
                board_msg,
                "
  输入 'accept <任务ID>' 即可接取."
                    .white()
                    .bold()
            );
            return ServerMessage::Info { payload: board_msg };
        }
    }

    msg
}
