#[cfg(test)]
mod tests {
    use crate::world::loader::load_all_data;
    use crate::world::player::{Player, PlayerQuestStatus};
    use crate::world::world_state::WorldState;
    use std::sync::Arc;
    use std::collections::HashMap;

    #[test]
    fn test_wild_zone_quest_and_respawn() {
        // 1. Initialize WorldState and load data
        let static_data = Arc::new(load_all_data("./data").expect("Failed to load world data"));
        let world_state = WorldState::new(static_data.clone());

        // 2. Create a Player
        let player_id = 1001u64;
        let mut player = Player::new(player_id, "测试剑客".to_string());

        // 3. Move player to deep_bamboo_1
        let room_id = "deep_bamboo_1";
        world_state.move_player_to_room(player_id, room_id);
        
        // Ensure room exists and player is there
        assert!(world_state.get_room(room_id).is_some());
        assert_eq!(world_state.get_player_room_id(player_id).unwrap(), room_id);

        // 4. Command 'accept q201'
        let quest_id = "q201";
        let _quest = static_data.quests.get(quest_id).expect("Quest q201 not found");
        
        // Simulate acceptance logic
        player.active_quests.push(PlayerQuestStatus {
            quest_id: quest_id.to_string(),
            current_step: 0,
            is_completed: false,
            kill_counts: HashMap::new(),
        });

        // 5. Verify q201 is in active_quests
        let active_q = player.active_quests.iter().find(|q| q.quest_id == quest_id);
        assert!(active_q.is_some());

        // 6 & 7. Simulate killing a 3002 (翠竹蛇) and trigger on_kill
        let monster_id = "3002";
        player.on_kill(monster_id, &static_data.quests);

        // 8. Check if q201 progress is 1/10
        let updated_q = player.active_quests.iter().find(|q| q.quest_id == quest_id).unwrap();
        let count = updated_q.kill_counts.get(monster_id).cloned().unwrap_or(0);
        assert_eq!(count, 1, "Kill count for 3002 should be 1");
        
        // 9. Simulate 120 seconds passing and call respawn
        // Clear NPCs in the room first to ensure respawn logic has work to do
        {
            let mut data = world_state.dynamic_data.lock().unwrap();
            data.npcs.retain(|_, npc| npc.current_room != room_id);
        }
        
        let initial_npc_count = world_state.get_npcs_in_room(room_id).len();
        assert_eq!(initial_npc_count, 0);

        // Trigger respawn logic (simulating global tick effect)
        world_state.respawn_monsters();

        // 10. Verify new monsters are spawned in the room
        let respawned_npcs = world_state.get_npcs_in_room(room_id);
        assert!(respawned_npcs.len() > 0, "Room should have respawned monsters");
        
        // Verify one of the monsters is from the allowed list for deep_bamboo
        let allowed_monsters = vec![3001, 3002, 3003, 2000]; // 2000 is QuestBoard
        let has_valid_monster = respawned_npcs.iter().any(|n| allowed_monsters.contains(&n.prototype_id));
        assert!(has_valid_monster);
    }
}