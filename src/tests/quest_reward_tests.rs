#[cfg(test)]
mod tests {
    use crate::world::loader::load_all_data;
    use crate::world::player::{Player, PlayerQuestStatus};
    use crate::world::world_state::WorldState;
    use crate::npc::Npc;
    use std::sync::Arc;
    use std::collections::HashMap;

    #[test]
    fn test_kill_quest_completion_and_reward() {
        // 1. Initialize world state and a player
        let static_data = Arc::new(load_all_data("./data").expect("Failed to load world data"));
        let world_state = WorldState::new(static_data.clone());
        
        let player_id = 999u64;
        let mut player = Player::new(player_id, "奖赏测试者".to_string());
        let initial_shell = player.wallet.shell;
        let initial_level = player.realm_sub_level;

        // 2. Manually add q201 to active_quests
        let quest_id = "q201".to_string();
        
        player.active_quests.push(PlayerQuestStatus {
            quest_id: quest_id.clone(),
            current_step: 0,
            is_completed: false,
            kill_counts: HashMap::new(),
        });

        // Move player to the room with the monster
        let room_id = "deep_bamboo_2";
        world_state.move_player_to_room(player_id, room_id, None);

        // 4. Loop 2 times to simulate 2 kills
        for i in 1..=2 {
            // Ensure there is a monster (3002) in the room
            let monster_proto_id = 3002u32;
            {
                let mut data = world_state.dynamic_data.lock().unwrap();
                // Clear existing npcs to control the environment
                data.npcs.retain(|_, n| n.current_room != room_id);
                
                // Spawn a fresh 3002
                let proto = static_data.npc_prototypes.get(&monster_proto_id).unwrap();
                let npc_id = data.next_npc_id;
                let npc = Npc::from_prototype(
                    npc_id,
                    monster_proto_id,
                    proto,
                    room_id.to_string(),
                );
                data.npcs.insert(npc_id, npc);
                data.next_npc_id += 1;
            }

            // Simulate the Attack logic (Targeting 3002)
            let npcs = world_state.get_npcs_in_room(room_id);
            let target_npc = npcs.iter().find(|n| n.prototype_id == monster_proto_id).unwrap();
            
            // Trigger kill logic
            let target_id_str = target_npc.prototype_id.to_string();
            let quest_msg = player.on_kill(&target_id_str, &static_data.quests);
            
            // Check progress message
            let expected = format!("{}/2", i);
            assert!(quest_msg.contains(&expected), "Message should contain progress info");

            // Remove the NPC instance (simulating death)
            {
                let mut data = world_state.dynamic_data.lock().unwrap();
                data.npcs.remove(&target_npc.instance_id);
            }

            // Immediate Reward Handling Logic (as implemented in handle_command)
            let mut completed_ids = Vec::new();
            for status in &player.active_quests {
                if status.is_completed {
                    completed_ids.push(status.quest_id.clone());
                }
            }

            for cid in completed_ids {
                if let Some(q) = static_data.quests.get(&cid) {
                    // Grant Reward
                    player.grant_reward(&q.rewards);
                    // Move to completed
                    player.completed_quests.insert(cid.clone());
                }
            }
            
            // Clean up active quests
            player.active_quests.retain(|q| !player.completed_quests.contains(&q.quest_id));
        }

        // 5. Verify Final State
        // q201 NO LONGER in active_quests
        assert!(!player.active_quests.iter().any(|q| q.quest_id == quest_id));
        
        // player.completed_quests contains q201
        assert!(player.completed_quests.contains(&quest_id));
        
        // player.wallet.shell has increased by 200
        assert_eq!(player.wallet.shell, initial_shell + 200);
        
        // player.realm_sub_level has increased (500 exp is enough to level up from 1 to 3)
        assert!(player.realm_sub_level > initial_level, "Player should have leveled up from quest reward");
        assert!(player.realm_sub_level >= 3, "500 EXP should bring player to at least Level 3");
    }
}
