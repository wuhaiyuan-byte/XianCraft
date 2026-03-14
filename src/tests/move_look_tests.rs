#[cfg(test)]
mod tests {
    use crate::world::world_loader::load_all_data;
    use crate::world::world_state::WorldState;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_move_updates_player_location() {
        let static_data = Arc::new(load_all_data("./data").expect("Failed to load world data"));
        let world_state = WorldState::new(static_data.clone());
        
        world_state.move_player_to_room(1, "genesis_altar", Some("玩家A".to_string()));
        
        let room_id = world_state.get_player_room_id(1);
        assert_eq!(room_id, Some("genesis_altar".to_string()));
        
        world_state.move_player_to_room(1, "village_square", Some("玩家A".to_string()));
        
        let new_room_id = world_state.get_player_room_id(1);
        assert_eq!(new_room_id, Some("village_square".to_string()));
        
        println!("✅ test_move_updates_player_location passed!");
    }

    #[tokio::test]
    async fn test_concurrent_world_access() {
        use std::sync::Arc;
        use tokio::task;
        
        let static_data = Arc::new(load_all_data("./data").expect("Failed to load world data"));
        let world_state = WorldState::new(static_data.clone());
        
        world_state.move_player_to_room(1, "genesis_altar", Some("玩家1".to_string()));
        world_state.move_player_to_room(2, "genesis_altar", Some("玩家2".to_string()));
        
        let ws1 = world_state.clone();
        let ws2 = world_state.clone();
        
        let handle1 = task::spawn(async move {
            for _ in 0..10 {
                ws1.move_player_to_room(1, "village_square", Some("玩家1".to_string()));
                ws1.move_player_to_room(1, "genesis_altar", Some("玩家1".to_string()));
            }
        });
        
        let handle2 = task::spawn(async move {
            for _ in 0..10 {
                ws2.move_player_to_room(2, "village_square", Some("玩家2".to_string()));
                ws2.move_player_to_room(2, "genesis_altar", Some("玩家2".to_string()));
            }
        });
        
        let _ = tokio::join!(handle1, handle2);
        
        println!("✅ test_concurrent_world_access passed!");
    }
}
