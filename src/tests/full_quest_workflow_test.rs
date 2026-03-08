#[cfg(test)]
mod tests {
    use crate::world::loader::load_all_data;
    use crate::world::player::{Player, PlayerQuestStatus};
    use crate::world::world_state::WorldState;
    use crate::world_model::Quest;
    use std::sync::Arc;
    use std::collections::HashMap;

    #[test]
    fn test_full_quest_system_flow() {
        // 1. Initialize world state and a test player at the starting room
        let static_data = Arc::new(load_all_data("./data").expect("Failed to load world data"));
        let world_state = WorldState::new(static_data.clone());
        
        let player_id = 999u64;
        let mut player = Player::new(player_id, "道号测试".to_string());
        
        let start_room = "genesis_altar";
        world_state.move_player_to_room(player_id, start_room);
        assert_eq!(world_state.get_player_room_id(player_id).unwrap(), start_room);

        let initial_shell = player.wallet.shell;
        let initial_level = player.realm_sub_level;

        // 2. Step 1 (Movement): Move player to deep_bamboo_1 via bamboo_forest
        world_state.move_player_to_room(player_id, "bamboo_forest");
        world_state.move_player_to_room(player_id, "deep_bamboo_1");
        let current_room = world_state.get_player_room_id(player_id).unwrap();
        assert_eq!(current_room, "deep_bamboo_1");

        // 3. Step 2 (Acceptance): Verify QuestBoard is present and accept q201
        let npcs_at_entrance = world_state.get_npcs_in_room(&current_room);
        let has_board = npcs_at_entrance.iter().any(|n| n.prototype_id == 2000);
        assert!(has_board, "QuestBoard (2000) must be present at deep_bamboo_1");

        let quest_id = "q201";
        let quest_proto = static_data.quests.get(quest_id).expect("Quest q201 must exist");
        
        let accepted = player.accept_quest(quest_proto);
        assert!(accepted, "Player should successfully accept q201");
        assert!(player.active_quests.iter().any(|q| q.quest_id == quest_id));

        // 4. Step 3 (Combat/Progress): Simulate 10 Attack commands on 3002 (翠竹蛇)
        let monster_id = "3002";
        // Move to a room where the snake actually spawns
        world_state.move_player_to_room(player_id, "deep_bamboo_2");
        let combat_room = "deep_bamboo_2";

        for i in 1..=10 {
            // Find an instance of 3002 in the room
            let target_instance_id = {
                let npcs = world_state.get_npcs_in_room(combat_room);
                npcs.iter()
                    .find(|n| n.prototype_id.to_string() == monster_id)
                    .map(|n| n.instance_id)
                    .expect(&format!("Could not find monster 3002 for kill #{}", i))
            };

            // Simulate the Attack logic: Remove NPC and trigger on_kill
            {
                let mut data = world_state.dynamic_data.lock().unwrap();
                data.npcs.remove(&target_instance_id);
            }
            
            let progress_msg = player.on_kill(monster_id, &static_data.quests);
            assert!(progress_msg.contains(&format!("{}/10", i)));

            // Verify NPC removed
            let npcs_after = world_state.get_npcs_in_room(combat_room);
            assert!(!npcs_after.iter().any(|n| n.instance_id == target_instance_id));

            // Call respawn to bring a new one back for the next iteration (unless it's the last kill)
            if i < 10 {
                world_state.respawn_monsters();
                let npcs_respawned = world_state.get_npcs_in_room(combat_room);
                // Note: Respawn might pick a different monster from the room list (3001 or 3002),
                // so we force-inject a 3002 if random selection didn't pick it to keep the test deterministic.
                if !npcs_respawned.iter().any(|n| n.prototype_id.to_string() == monster_id) {
                    let mut data = world_state.dynamic_data.lock().unwrap();
                    let prototype = static_data.npc_prototypes.get(&3002).unwrap();
                    let npc_id = data.next_npc_id;
                    let npc = crate::npc::Npc::from_prototype(
                        npc_id,
                        3002,
                        prototype,
                        combat_room.to_string(),
                    );
                    data.npcs.insert(npc_id, npc);
                    data.next_npc_id += 1;
                }
            }
        }

        // 5. Step 4 (Completion): Verify is_completed is true
        let final_status = player.active_quests.iter().find(|q| q.quest_id == quest_id).unwrap();
        assert!(final_status.is_completed, "Quest q201 should be marked as completed after 10 kills");

        // 6. Step 5 (Rewards): Call grant_reward and verify state
        let reward_msg = player.grant_reward(&quest_proto.rewards);
        assert!(reward_output_is_valid(&reward_msg));
        
        assert!(player.wallet.shell > initial_shell, "Player shell count should increase");
        assert!(player.realm_sub_level > initial_level, "Player should have leveled up from exp");

        // 7. Step 6 (Persistence/Retain logic):
        // Simulate a "Go" or movement check. 
        // In the real handle_command, we'd call retain. We'll simulate that here.
        player.active_quests.retain(|q| {
            if let Some(qd) = static_data.quests.get(&q.quest_id) {
                // The fixed logic: kill quests only removed if completed.
                // Note: In our test, we just granted rewards, but usually retain happens 
                // BEFORE or DURING the process. Let's verify a completed quest isn't 
                // wiped by the old "steps.len()" bug.
                if qd.quest_type == "kill" {
                    return !q.is_completed; // Remove only if we are done
                }
                if q.current_step as usize >= qd.steps.len() {
                    return false;
                }
            }
            true
        });

        // After reward/cleanup, it should be gone from active
        assert!(!player.active_quests.iter().any(|q| q.quest_id == quest_id));
    }

    fn reward_output_is_valid(msg: &str) -> bool {
        msg.contains("任务圆满完成") && msg.contains("修为") && msg.contains("灵贝")
    }
}