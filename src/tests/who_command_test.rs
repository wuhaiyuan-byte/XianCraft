#[cfg(test)]
mod tests {
    use crate::command::parse;
    use crate::generate_who_list;
    use crate::handle_command;
    use crate::world::loader::load_all_data;
    use crate::world::player::Player;
    use crate::world::world_state::WorldState;
    use crate::{AppState, PlayerSession};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use strip_ansi_escapes::strip;
    use tokio::sync::mpsc;

    fn create_player_session(id: u64, name: &str, realm_level: u16, realm_sub_level: u16, sect: Option<String>, is_resting: bool, sender: mpsc::Sender<axum::extract::ws::Message>) -> PlayerSession {
        let mut player = Player::new(id, name.to_string());
        player.realm_level = realm_level;
        player.realm_sub_level = realm_sub_level;
        player.sect = sect;
        player.is_resting = is_resting;
        PlayerSession {
            player,
            user_id: Some(name.to_string()),
            sender,
        }
    }

    #[tokio::test]
    async fn test_who_command_shows_all_online_players() {
        let static_data = Arc::new(load_all_data("./data").expect("Failed to load world data"));
        let world_state = WorldState::new(static_data.clone());
        let app_state = Arc::new(AppState {
            world_state,
            player_sessions: Mutex::new(HashMap::new()),
        });

        let (sender1, _receiver1) = mpsc::channel(100);
        let (sender2, _receiver2) = mpsc::channel(100);
        let (sender3, _receiver3) = mpsc::channel(100);

        let p1 = create_player_session(1, "玩家一", 1, 5, Some("青云宗".to_string()), false, sender1);
        let p2 = create_player_session(2, "玩家二", 1, 8, Some("天剑门".to_string()), false, sender2);
        let p3 = create_player_session(3, "玩家三", 2, 3, Some("御兽园".to_string()), false, sender3);

        {
            let mut sessions = app_state.player_sessions.lock().unwrap();
            sessions.insert(1, p1);
            sessions.insert(2, p2);
            sessions.insert(3, p3);
        }

        let output = generate_who_list(&app_state, false);
        let uncolored = String::from_utf8(strip(output.as_bytes())).unwrap();

        assert!(uncolored.contains("玩家一"), "Should contain player 1 name");
        assert!(uncolored.contains("玩家二"), "Should contain player 2 name");
        assert!(uncolored.contains("玩家三"), "Should contain player 3 name");
        assert!(uncolored.contains("3位"), "Should show 3 players online");
        
        println!("WHO OUTPUT:\n{}", output);
    }

    #[tokio::test]
    async fn test_who_command_sorts_by_realm_level_descending() {
        let static_data = Arc::new(load_all_data("./data").expect("Failed to load world data"));
        let world_state = WorldState::new(static_data.clone());
        let app_state = Arc::new(AppState {
            world_state,
            player_sessions: Mutex::new(HashMap::new()),
        });

        let (sender1, _receiver1) = mpsc::channel(100);
        let (sender2, _receiver2) = mpsc::channel(100);
        let (sender3, _receiver3) = mpsc::channel(100);

        let p1 = create_player_session(1, "低级修士", 1, 3, None, false, sender1);
        let p2 = create_player_session(2, "中级修士", 2, 5, None, false, sender2);
        let p3 = create_player_session(3, "高级修士", 3, 1, None, false, sender3);

        {
            let mut sessions = app_state.player_sessions.lock().unwrap();
            sessions.insert(1, p1);
            sessions.insert(2, p2);
            sessions.insert(3, p3);
        }

        let output = generate_who_list(&app_state, false);
        let uncolored = String::from_utf8(strip(output.as_bytes())).unwrap();
        
        println!("SORTED WHO OUTPUT:\n{}", uncolored);
        
        let jindan_pos = uncolored.find("金丹");
        let zhuji_pos = uncolored.find("筑基");
        let lianqi_pos = uncolored.find("炼气");
        
        assert!(jindan_pos.is_some() && zhuji_pos.is_some() && lianqi_pos.is_some());
        
        let j = jindan_pos.unwrap();
        let z = zhuji_pos.unwrap();
        let l = lianqi_pos.unwrap();
        
        eprintln!("Positions:金丹={}, 筑基={}, 炼气={}", j, z, l);
        
        assert!(j < z, "金丹 should come before 筑基, got j={}, z={}", j, z);
        assert!(z < l, "筑基 should come before 炼气, got z={}, l={}", z, l);
    }

    #[tokio::test]
    async fn test_who_command_reflects_resting_status() {
        let static_data = Arc::new(load_all_data("./data").expect("Failed to load world data"));
        let world_state = WorldState::new(static_data.clone());
        let app_state = Arc::new(AppState {
            world_state,
            player_sessions: Mutex::new(HashMap::new()),
        });

        let (sender1, _receiver1) = mpsc::channel(100);
        
        let p1 = create_player_session(1, "打坐修士", 1, 5, None, true, sender1);

        {
            let mut sessions = app_state.player_sessions.lock().unwrap();
            sessions.insert(1, p1);
        }

        let output = generate_who_list(&app_state, false);
        let uncolored = String::from_utf8(strip(output.as_bytes())).unwrap();
        
        assert!(uncolored.contains("打坐中"), "Resting player should show '打坐中'");
        
        println!("RESTING STATUS OUTPUT:\n{}", output);
        
        {
            let mut sessions = app_state.player_sessions.lock().unwrap();
            if let Some(session) = sessions.get_mut(&1) {
                session.player.is_resting = false;
            }
        }
        
        let output2 = generate_who_list(&app_state, false);
        let uncolored2 = String::from_utf8(strip(output2.as_bytes())).unwrap();
        
        assert!(uncolored2.contains("游历中"), "Non-resting player should show '游历中'");
        
        println!("NON-RESTING STATUS OUTPUT:\n{}", output2);
    }
}

