use crate::world::world_player::Player;
use crate::ServerMessage;
use colored::*;
use rand::seq::SliceRandom;

pub fn handle_rest(player: &mut Player) -> ServerMessage {
    player.is_resting = !player.is_resting;
    if player.is_resting {
        ServerMessage::Info {
            payload: "你开始原地休息，逐渐恢复精力。".to_string(),
        }
    } else {
        ServerMessage::Info {
            payload: "你站了起来，感觉精力充沛了一些。".to_string(),
        }
    }
}

pub fn handle_work(player: &mut Player, current_room_id: &str) -> Vec<ServerMessage> {
    let mut messages = Vec::new();

    if current_room_id != "bamboo_forest" && current_room_id != "spirit_spring" {
        messages.push(ServerMessage::Error {
            payload: "这里似乎没有什么值得你劳作的地方，换个环境试试？".to_string(),
        });
        return messages;
    }

    if !player.consume_stamina(15) {
        messages.push(ServerMessage::Error {
            payload: "你已经筋疲力尽了，稍微休息（rest）一下吧。".to_string(),
        });
        return messages;
    }

    player.wallet.shell += 20;
    player.exp += 5;
    player.potential += 2;

    let pool = [
        "你抡起斧头劈向枯木，震得虎口生疼，但隐约间你捕捉到了风的律动。",
        "汗水顺着脸颊流下，你进入了一种奇妙的节奏，呼吸逐渐与竹林的沙沙声同步。",
        "每一次挥砍都带起片片竹叶，你感到体内有一丝微弱的气流正随着动作缓缓升起。",
    ];
    let mut rng = rand::thread_rng();
    let msg = pool.choose(&mut rng).unwrap().to_string();
    messages.push(ServerMessage::Description { payload: msg });
    messages.push(ServerMessage::Info {
        payload: "获得奖励：灵贝+20，修为+5，潜能+2"
            .green()
            .bold()
            .to_string(),
    });

    let mut q102_finished = false;
    for status in &mut player.active_quests {
        if status.quest_id == "q102" && status.current_step == 2 {
            let count = player
                .quest_counts
                .entry("q102_work".to_string())
                .or_insert(0);
            *count += 1;
            if *count >= 5 {
                status.current_step += 1;
                q102_finished = true;
            }
        }
    }
    if q102_finished {
        messages.push(ServerMessage::Description { payload: "【机缘】随着最后一斧劈下，你感到一股清凉的气流顺着指尖流向全身。你对天地的感悟达到了新的高度！请回广场向村长报告。".magenta().bold().to_string() });
    }

    messages
}

pub fn handle_inventory(
    player: &Player,
    world: &crate::world::world_state::WorldState,
) -> ServerMessage {
    if player.inventory.is_empty() {
        ServerMessage::Info {
            payload: "你两手空空。".to_string(),
        }
    } else {
        let mut inv_text = format!("{}\n", "你身上带着：".yellow().bold());
        for item_id in &player.inventory {
            let item_name = world
                .static_data
                .item_prototypes
                .get(item_id)
                .map(|p| p.name.clone())
                .unwrap_or_else(|| "未知物品".to_string());
            inv_text.push_str(&format!("- {}\n", item_name));
        }
        ServerMessage::Description { payload: inv_text }
    }
}

pub fn handle_score(player: &Player, config: &crate::world_model::WorldConfig) -> ServerMessage {
    let score_str = player.get_score_string(config);
    ServerMessage::Description { payload: score_str }
}
