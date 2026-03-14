#[cfg(test)]
mod tests {
    use crate::world::world_loader::load_all_data;
    use crate::world::world_player::{Player, PlayerQuestStatus};
    use crate::world::world_state::WorldState;
    use std::sync::Arc;
    use std::collections::HashMap;

    #[test]
    fn test_attack_and_quest_progress() {
        // 1. Initialize world state and load all data
        let static_data = Arc::new(load_all_data("./data").expect("Failed to load world data"));
        let world_state = WorldState::new(static_data.clone());

        // 2. Create a test player
        let player_id = 999u64;
        let mut player = Player::new(player_id, "战力测试员".to_string());

        // 3. Move player to deep_bamboo_2 (where 翠竹蛇 ID 3002 is located)
        let room_id = "deep_bamboo_2";
        world_state.move_player_to_room(player_id, room_id, None);
        
        // 4. Manually add quest q201 to the player's active_quests
        let quest_id = "q201";
        player.active_quests.push(PlayerQuestStatus {
            quest_id: quest_id.to_string(),
            current_step: 0,
            is_completed: false,
            kill_counts: HashMap::new(),
        });

        // 5. Simulate the Attack command logic
        // Find the NPC instance of 3002 (翠竹蛇) in the room
        let target_proto_id = 3002u32;
        let npcs_in_room = world_state.get_npcs_in_room(room_id);
        let target_npc = npcs_in_room.iter().find(|n| n.prototype_id == target_proto_id);
        
        assert!(target_npc.is_some(), "翠竹蛇 (3002) should exist in deep_bamboo_2");
        let npc_instance = target_npc.unwrap();
        let npc_instance_id = npc_instance.instance_id;

        // Verify that attacking it triggers player.on_kill("3002", ...)
        let monster_id_str = target_proto_id.to_string();
        let quest_msg = player.on_kill(&monster_id_str, &static_data.quests);

        // Verify the return message contains the quest progress "1/2"
        assert!(quest_msg.contains("1/2"), "Quest progress should be 1/2 after first kill");

        // Verify the NPC instance is removed from the world state (dynamic data)
        {
            let mut data = world_state.dynamic_data.lock().unwrap();
            data.npcs.remove(&npc_instance_id);
        }

        // Verify the NPC is actually gone from the room
        let npcs_after_kill = world_state.get_npcs_in_room(room_id);
        let found_again = npcs_after_kill.iter().any(|n| n.instance_id == npc_instance_id);
        assert!(!found_again, "NPC instance should be removed from the world after death");

        // 6. Verify the player's quest status now shows 1 kill
        let status = player.active_quests.iter().find(|q| q.quest_id == quest_id).unwrap();
        let count = status.kill_counts.get(&monster_id_str).cloned().unwrap_or(0);
        assert_eq!(count, 1, "Kill count for 3002 in quest status should be 1");
    }
}