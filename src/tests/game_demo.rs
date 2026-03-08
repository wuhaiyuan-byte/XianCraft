#[cfg(test)]
mod tests {
    use crate::world::loader::load_all_data;
    use crate::world::player::{Player, PlayerQuestStatus};
    use crate::world::world_state::WorldState;
    use crate::npc::Npc;
    use std::sync::Arc;
    use std::collections::HashMap;
    use std::thread::sleep;
    use std::time::Duration;

    fn display(msg: &str) {
        println!("{}", msg);
        // Add a small delay to make it readable like a real game
        sleep(Duration::from_millis(300));
    }

    #[test]
    fn run_visual_demo() {
        // 1. Initialize world state and load all data
        let static_data = Arc::new(load_all_data("./data").expect("Failed to load world data"));
        let world_state = WorldState::new(static_data.clone());

        // 2. Create a Player
        let player_id = 777u64;
        let mut player = Player::new(player_id, "云中鹤".to_string());

        display("\n\x1b[1;35m====================================================\x1b[0m");
        display("\x1b[1;33m          仙径尘缘 (MUD) - 自动化演示启动           \x1b[0m");
        display("\x1b[1;35m====================================================\x1b[0m\n");

        // 3. MOVE: Show the entrance
        let room_id = "deep_bamboo_1";
        world_state.move_player_to_room(player_id, room_id);
        let room = world_state.get_room(room_id).unwrap();
        
        display(&format!("\x1b[1;32m{}\x1b[0m", room.name));
        display(&format!("\x1b[0;37m{}\x1b[0m", room.description));
        display("\x1b[1;34m● 告示牌, 灵蝶\x1b[0m");
        display("\x1b[1;36m出口: [north, south]\x1b[0m\n");

        // 4. LOOK: Show the QuestBoard content
        display("\x1b[1;33m你仔细观察告示牌上的字迹...\x1b[0m");
        display("\n\x1b[1;33m告示牌上贴着以下悬赏：\x1b[0m");
        display("- [q201] 清缴翠竹蛇 (目标: 10只)");
        display("\x1b[1;37m输入 'accept <任务ID>' 即可接取。\x1b[0m\n");

        // 5. ACCEPT: Show the acceptance notification
        let quest_id = "q201";
        let quest = static_data.quests.get(quest_id).unwrap();
        player.active_quests.push(PlayerQuestStatus {
            quest_id: quest_id.to_string(),
            current_step: 0,
            is_completed: false,
            kill_counts: HashMap::new(),
        });
        display(&format!("\x1b[1;33m[任务接取] 你接取了任务：{}。输入 'qs' 可查看详细进度。\x1b[0m\n", quest.name));

        // 6. COMBAT: Simulate 3 kills
        let monster_id = "3002";
        for i in 1..=3 {
            display(&format!("\x1b[1;31m你对着 翠竹蛇 发起猛攻，几个回合后将其击败了！\x1b[0m"));
            let msg = player.on_kill(monster_id, &static_data.quests);
            display(&msg);
        }

        // 7. STATUS: Show the full Score panel
        display("\n\x1b[1;33m你查看了自己的修行进度：\x1b[0m");
        display(&player.get_score_string(&static_data.config));

        // 8. REWARD: Fast-forward to 10 kills
        display("\n\x1b[1;35m>>> 经过一番激战，你终于完成了清剿任务... <<<\x1b[0m\n");
        for _ in 4..=10 {
            player.on_kill(monster_id, &static_data.quests);
        }
        
        // Final completion and reward message
        let status = player.active_quests.iter().find(|q| q.quest_id == quest_id).unwrap();
        if status.is_completed {
            display("\x1b[1;33m你已达成任务“清缴翠竹蛇”的目标！\x1b[0m");
            let reward_msg = player.grant_reward(&quest.rewards);
            display(&reward_msg);
        }

        display("\n\x1b[1;35m====================================================\x1b[0m");
        display("\x1b[1;32m演示圆满结束。修为大涨！\x1b[0m");
        display("\x1b[1;35m====================================================\x1b[0m\n");
    }
}