#[cfg(test)]
mod tests {
    use crate::command::parse;
    use crate::handle_command;
    use crate::world::loader::load_all_data;
    use crate::world::player::Player;
    use crate::world::world_state::WorldState;
    use crate::{AppState, PlayerSession};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use strip_ansi_escapes::strip;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_help_command_sends_correct_message() {
        // 1. Initialize world state and app state directly, just like other tests in `src/tests`
        let static_data = Arc::new(load_all_data("./data").expect("Failed to load world data"));
        let world_state = WorldState::new(static_data.clone());
        let app_state = Arc::new(AppState {
            world_state,
            player_sessions: Mutex::new(HashMap::new()),
        });

        // 2. Create a test player and a mock channel to receive messages
        let player_id = 1u64;
        let (sender, mut receiver) = mpsc::channel(100);
        let player_session = PlayerSession {
            player: Player::new(player_id, "帮助测试员".to_string()),
            user_id: Some("帮助测试员".to_string()),
            sender: sender.clone(),
        };
        app_state.player_sessions.lock().unwrap().insert(player_id, player_session);
        app_state.world_state.move_player_to_room(player_id, "genesis_altar");

        // 3. Parse the command we want to test
        let command = parse("help");

        // 4. Directly call the (private) command handler, as we are in an inline test module
        handle_command(command, player_id, app_state.clone(), sender).await;

        // 5. Assert that the handler sent the correct help message to our mock channel
        let mut help_response_found = false;
        if let Some(msg) = receiver.recv().await {
            if let axum::extract::ws::Message::Text(text) = msg {
                let v: serde_json::Value = serde_json::from_str(&text)
                    .expect("Handler sent a message that was not valid JSON");

                assert_eq!(v["type"], "Info", "Message type should be 'Info'");

                if let Some(payload) = v["payload"].as_str() {
                    // --- ADDED FOR VISUAL OUTPUT ---
                    println!("\n--- Captured Help Command Output ---\n{}", payload);
                    // --------------------------------
                    
                    let uncolored_payload = String::from_utf8(strip(payload.as_bytes()))
                        .expect("Payload contained invalid UTF-8 after stripping ANSI codes");

                    if uncolored_payload.contains("可用指令 (Commands)") {
                        assert!(uncolored_payload.contains("look"));
                        assert!(uncolored_payload.contains("go <方向>"));
                        println!("test_help_command PASSED: Correct help message received.");
                        help_response_found = true;
                    }
                }
            }
        }

        assert!(help_response_found, "Did not receive the expected help info message from the handler.");
    }
}
