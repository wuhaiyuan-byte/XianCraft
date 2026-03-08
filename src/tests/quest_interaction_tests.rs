#[cfg(test)]
mod tests {
    use crate::world::loader::load_all_data;
    use crate::world::player::{Player, PlayerQuestStatus};
    use crate::world::world_state::WorldState;
    use crate::world_model::Quest;
    use std::sync::Arc;
    use std::collections::HashMap;

    #[test]
    fn test_comprehensive_quest_workflow() {
        // 1. Initialize world state and load all data
        let static_data = Arc::new(load_all_data("./data").expect("Failed to load world data"));
        let world_state = WorldState::new(static_data.clone());

        // 2. Create a test player
        let player_id = 888u64;
        let mut player = Player::new(player_id, "剑修测试者".to_string());
        let initial_realm_sub_level = player.realm_sub_level;
        let initial_exp = player.exp;
        let initial_shell = player.wallet.shell;

        // 3. Simulate moving the player to 'deep_bamboo_1'
        let room_id = "deep_bamboo_1";
        world_state.move_player_to_room(player_id, room_id);
        assert_eq!(world_state.get_player_room_id(player_id).unwrap(), room_id);

        // 4. Verify quest board and quest availability
        let npcs = world_state.get_npcs_in_room(room_id);
        let has_board = npcs.iter().any(|n| n.prototype_id == 2000);
        assert!(has_board, "QuestBoard (2000) should be in deep_bamboo_1");

        let quest_id = "q201";
        let quest = static_data.quests.get(quest_id).expect("Quest q201 must exist in registry");
        assert_eq!(quest.quest_type, "kill");
        assert_eq!(quest.target_id, "3002");

        // 5. Simulate 'accept q201'
        let accepted = player.accept_quest(quest);
        assert!(accepted, "Player should be able to accept q201");
        
        let active_q = player.active_quests.iter().find(|q| q.quest_id == quest_id);
        assert!(active_q.is_some(), "q201 should be in player's active_quests");

        // 6. Simulate killing the first '3002' (翠竹蛇)
        let monster_id = "3002";
        let kill_msg_1 = player.on_kill(monster_id, &static_data.quests);
        
        assert!(kill_msg_1.contains("1/10"), "Progress message should show 1/10");
        let status = player.active_quests.iter().find(|q| q.quest_id == quest_id).unwrap();
        assert_eq!(*status.kill_counts.get(monster_id).unwrap(), 1);
        assert!(!status.is_completed, "Quest should not be completed yet");

        // 7. Simulate killing 9 more '3002' to complete the quest (Total 10)
        let mut final_msg = String::new();
        for _ in 0..9 {
            final_msg = player.on_kill(monster_id, &static_data.quests);
        }

        assert!(final_msg.contains("10/10"), "Progress message should show 10/10");
        assert!(final_msg.contains("已达成任务"), "Completion message should be triggered");
        
        let status_final = player.active_quests.iter().find(|q| q.quest_id == quest_id).unwrap();
        assert!(status_final.is_completed, "Quest status should be marked as completed");

        // 8. Simulate calling 'grant_reward'
        let reward_output = player.grant_reward(&quest.rewards);
        
        assert!(reward_output.contains("任务圆满完成"), "Reward message should be returned");
        assert!(player.realm_sub_level > initial_realm_sub_level || player.exp > initial_exp, "Player should have gained experience or leveled up");
        assert!(player.wallet.shell > initial_shell, "Player Money should have increased");
        
        // Final sanity check on values
        if let Some(expected_shell) = quest.rewards.shell {
            assert_eq!(player.wallet.shell, initial_shell + expected_shell);
        }
    }
}