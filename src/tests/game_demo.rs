#[cfg(test)]
mod tests {
    use crate::world::loader::load_all_data;
    use crate::world::player::{Player, PlayerQuestStatus};
    use crate::world::world_state::WorldState;
    use std::sync::Arc;
    use std::collections::HashMap;
    use std::thread::sleep;
    use std::time::Duration;
    use colored::*;

    fn display(msg: &str) {
        println!("{}", msg);
        // Add a small delay to make it readable like a real game
        sleep(Duration::from_millis(300));
    }

    #[test]
    fn run_visual_demo() {
        // Force color output for the test environment, in case it's not a TTY
        control::set_override(true);

        // 1. Initialize world state and load all data
        let static_data = Arc::new(load_all_data("./data").expect("Failed to load world data"));
        let world_state = WorldState::new(static_data.clone());

        // 2. Create a Player
        let player_id = 777u64;
        let mut player = Player::new(player_id, "云中鹤".to_string());

        display(&format!("\n{}", "====================================================".truecolor(255, 105, 180)));
        display(&format!("{}", "          仙径尘缘 (MUD) - 自动化演示启动           ".yellow().bold()));
        display(&format!("{}\n", "====================================================".truecolor(255, 105, 180)));

        // 3. MOVE: Show the entrance
        let room_id = "deep_bamboo_1";
        world_state.move_player_to_room(player_id, room_id, None);
        let room = world_state.get_room(room_id).unwrap();
        
        display(&room.name.green().bold().to_string());
        display(&room.description.white().to_string());
        display(&"● 告示牌, 灵蝶".blue().bold().to_string());
        display(&format!("{}\n", "出口: [north, south]".cyan().bold()));

        // 4. LOOK: Show the QuestBoard content
        display(&"你仔细观察告示牌上的字迹...".yellow().bold().to_string());
        display(&format!("\n{}", "告示牌上贴着以下悬赏：".yellow().bold()));
        display("- [q201] 清缴翠竹蛇 (目标: 10只)");
        display(&format!("{}\n", "输入 'accept <任务ID>' 即可接取。".white().bold()));

        // 5. ACCEPT: Show the acceptance notification
        let quest_id = "q201";
        let quest = static_data.quests.get(quest_id).unwrap();
        player.active_quests.push(PlayerQuestStatus {
            quest_id: quest_id.to_string(),
            current_step: 0,
            is_completed: false,
            kill_counts: HashMap::new(),
        });
        display(&format!("{}\n", format!("[任务接取] 你接取了任务：{}。输入 'qs' 可查看详细进度。", quest.name).yellow().bold()));

        // 6. COMBAT: Simulate 3 kills
        let monster_id = "3002";
        for _ in 1..=3 {
            display(&"你对着 翠竹蛇 发起猛攻，几个回合后将其击败了！".red().bold().to_string());
            let msg = player.on_kill(monster_id, &static_data.quests);
            display(&msg);
        }

        // 7. STATUS: Show the full Score panel
        display(&format!("\n{}", "你查看了自己的修行进度：".yellow().bold()));
        display(&player.get_score_string(&static_data.config));

        // 8. REWARD: Fast-forward to 10 kills
        display(&format!("\n{}", ">>> 经过一番激战，你终于完成了清剿任务... <<<".truecolor(255, 105, 180).bold()));
        for _ in 4..=10 {
            player.on_kill(monster_id, &static_data.quests);
        }
        
        // Final completion and reward message
        let status = player.active_quests.iter().find(|q| q.quest_id == quest_id).unwrap();
        if status.is_completed {
            display(&"你已达成任务“清缴翠竹蛇”的目标！".yellow().bold().to_string());
            let reward_msg = player.grant_reward(&quest.rewards);
            display(&reward_msg);
        }

        display(&format!("\n{}", "====================================================".truecolor(255, 105, 180)));
        display(&"演示圆满结束。修为大涨！".green().bold().to_string());
        display(&format!("{}\n", "====================================================".truecolor(255, 105, 180)));
    }
}
