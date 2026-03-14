use crate::npc::Npc;
use crate::world::world_state::WorldState;
use crate::AppState;
use colored::*;
use std::sync::Arc;

pub fn build_welcome_message() -> String {
    let pink = (255, 105, 180);
    let purple = (218, 112, 214);
    let white = (255, 255, 255);

    let line1 = format!(
        "      {}  {}  {}",
        "✧･ﾟ: *✧･ﾟ:*".truecolor(pink.0, pink.1, pink.2),
        "ଘ(◕‿◕✿)ଓ".truecolor(purple.0, purple.1, purple.2),
        "*:･ﾟ✧*:･ﾟ✧".truecolor(pink.0, pink.1, pink.2)
    );

    let line2 = format!(
        "    {}",
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".truecolor(purple.0, purple.1, purple.2)
    );

    let line3 = format!(
        "       {} {}",
        "(つ◕ヮ◕)つ".truecolor(pink.0, pink.1, pink.2),
        "⚔️  剑 灵 少 女 の 招 待  ⚔️"
            .bold()
            .truecolor(white.0, white.1, white.2)
    );

    let line4 = line2.clone();

    let line6 = format!(
        "      {}",
        "“欧尼酱！快握紧这把灵剑，一起踏上登仙之路吧~”"
            .bold()
            .truecolor(pink.0, pink.1, pink.2)
    );

    let line8 = format!(
        "             {}  {}  {}  {}  {}",
        "✦".truecolor(purple.0, purple.1, purple.2),
        "✧".truecolor(pink.0, pink.1, pink.2),
        "(ﾉ◕ヮ◕)ﾉ*:･ﾟ✧".truecolor(white.0, white.1, white.2),
        "✧".truecolor(pink.0, pink.1, pink.2),
        "✦".truecolor(purple.0, purple.1, purple.2)
    );

    format!(
        "{}
{}
{}
{}
{}

{}

{}",
        line1, line2, line3, line4, line6, line8, ""
    )
}

pub fn realm_level_to_name(level: u16, sub_level: u16) -> String {
    match level {
        1 => format!(
            "炼气{}层",
            match sub_level {
                1 => "一",
                2 => "二",
                3 => "三",
                4 => "四",
                5 => "五",
                6 => "六",
                7 => "七",
                8 => "八",
                9 => "九",
                _ => "?",
            }
        ),
        2 => format!(
            "筑基{}层",
            match sub_level {
                1 => "一",
                2 => "二",
                3 => "三",
                4 => "四",
                5 => "五",
                6 => "六",
                7 => "七",
                8 => "八",
                9 => "九",
                _ => "?",
            }
        ),
        3 => format!(
            "金丹{}层",
            match sub_level {
                1 => "一",
                2 => "二",
                3 => "三",
                4 => "四",
                5 => "五",
                6 => "六",
                7 => "七",
                8 => "八",
                9 => "九",
                _ => "?",
            }
        ),
        4 => format!(
            "元婴{}层",
            match sub_level {
                1 => "一",
                2 => "二",
                3 => "三",
                4 => "四",
                5 => "五",
                6 => "六",
                7 => "七",
                8 => "八",
                9 => "九",
                _ => "?",
            }
        ),
        5 => format!(
            "化神{}层",
            match sub_level {
                1 => "一",
                2 => "二",
                3 => "三",
                4 => "四",
                5 => "五",
                6 => "六",
                7 => "七",
                8 => "八",
                9 => "九",
                _ => "?",
            }
        ),
        _ => format!("境界{}", level),
    }
}

fn pad_to_width(s: &str, width: usize) -> String {
    let char_count = s.chars().count();
    if char_count >= width {
        return s.to_string();
    }
    let padding = width - char_count;
    let left = padding / 2;
    let right = padding - left;
    format!("{}{}{}", " ".repeat(left), s, " ".repeat(right))
}

pub fn parse_realm_level(realm: &str) -> (u16, u16) {
    let level = if realm.starts_with("炼气") {
        1
    } else if realm.starts_with("筑基") {
        2
    } else if realm.starts_with("金丹") {
        3
    } else if realm.starts_with("元婴") {
        4
    } else if realm.starts_with("化神") {
        5
    } else {
        0
    };

    let sub = if realm.contains("一") {
        1
    } else if realm.contains("二") {
        2
    } else if realm.contains("三") {
        3
    } else if realm.contains("四") {
        4
    } else if realm.contains("五") {
        5
    } else if realm.contains("六") {
        6
    } else if realm.contains("七") {
        7
    } else if realm.contains("八") {
        8
    } else if realm.contains("九") {
        9
    } else {
        0
    };

    (level, sub)
}

pub fn get_full_room_description(
    room_id: &str,
    world_state: &WorldState,
    other_players: Vec<String>,
    npcs_in_room: Vec<Npc>,
    room_items: Vec<u32>,
) -> String {
    if let Some(room) = world_state.get_room(room_id) {
        let mut full_desc = format!(
            "{}
{}",
            room.name.cyan().bold(),
            room.description
        );

        if !other_players.is_empty() {
            let player_list: Vec<String> = other_players
                .iter()
                .map(|n| n.green().to_string())
                .collect();
            full_desc.push_str(&format!(
                "
你在此处看到了：{}",
                player_list.join(", ")
            ));
        }

        if !npcs_in_room.is_empty() {
            let npc_names: Vec<String> = npcs_in_room
                .iter()
                .map(|npc| npc.name.green().to_string())
                .collect();
            full_desc.push_str(&format!(
                "
● {}",
                npc_names.join(", ")
            ));
        }

        if !room_items.is_empty() {
            let item_names: Vec<String> = room_items
                .iter()
                .filter_map(|id| world_state.static_data.item_prototypes.get(id))
                .map(|item| item.name.clone())
                .collect();
            if !item_names.is_empty() {
                full_desc.push_str(&format!(
                    "
{}",
                    item_names.join(", ")
                ));
            }
        }

        if !room.exits.is_empty() {
            let exit_keys: Vec<String> = room.exits.keys().cloned().collect();
            full_desc.push_str(&format!(
                "
{}",
                format!("出口: [{}]", exit_keys.join(", ")).white()
            ));
        }

        full_desc
    } else {
        "你身处一片虚无之中。".to_string()
    }
}

pub fn generate_who_list(state: &Arc<AppState>, use_color: bool) -> String {
    let players_data: Vec<(String, u16, u16, String, u64, bool)> = {
        let sessions = state.player_sessions.lock().unwrap();
        sessions
            .values()
            .filter(|s| s.user_id.is_some())
            .map(|s| {
                (
                    s.player.name.clone(),
                    s.player.realm_level,
                    s.player.realm_sub_level,
                    s.player.sect.clone().unwrap_or_else(|| "无".to_string()),
                    s.player.id,
                    s.player.is_resting,
                )
            })
            .collect()
    };

    let world = &state.world_state;

    let mut players: Vec<(String, String, String, String)> = players_data
        .into_iter()
        .map(
            |(name, realm_level, realm_sub_level, sect, player_id, is_resting)| {
                let realm = realm_level_to_name(realm_level, realm_sub_level);
                let status = {
                    let npcs = world
                        .get_npcs_in_room(&world.get_player_room_id(player_id).unwrap_or_default());
                    let in_combat = npcs.iter().any(|npc| {
                        if let Some(proto) = world.static_data.npc_prototypes.get(&npc.prototype_id)
                        {
                            (proto.ai == "monster"
                                || !proto.flags.contains(&"friendly".to_string()))
                                && npc.combat_target == Some(player_id)
                        } else {
                            false
                        }
                    });
                    if in_combat {
                        "战斗中".to_string()
                    } else if is_resting {
                        "打坐中".to_string()
                    } else {
                        "游历中".to_string()
                    }
                };
                (name, realm, sect, status)
            },
        )
        .collect();

    players.sort_by(|a, b| {
        let a_level = parse_realm_level(&a.1);
        let b_level = parse_realm_level(&b.1);
        b_level.cmp(&a_level)
    });

    let count = players.len();

    let name_width = 16;
    let realm_width = 16;
    let sect_width = 12;
    let status_width = 12;

    let header_line = format!(
        "┌{}┬{}┬{}┬{}┐",
        "─".repeat(name_width),
        "─".repeat(realm_width),
        "─".repeat(sect_width),
        "─".repeat(status_width)
    );
    let sep_line = format!(
        "├{}┼{}┼{}┼{}┤",
        "─".repeat(name_width),
        "─".repeat(realm_width),
        "─".repeat(sect_width),
        "─".repeat(status_width)
    );
    let footer_line = format!(
        "└{}┴{}┴{}┴{}┘",
        "─".repeat(name_width),
        "─".repeat(realm_width),
        "─".repeat(sect_width),
        "─".repeat(status_width)
    );

    let mut output = String::new();

    if use_color {
        output.push_str(&format!("{}\n", "【 仙 界 同 道 】".magenta().bold()));
        output.push_str(&format!("{}\n", header_line.truecolor(180, 100, 200)));
        output.push_str(&format!(
            "│{}│{}│{}│{}│\n",
            pad_to_width("姓 名", name_width - 2).white(),
            pad_to_width("境 界", realm_width - 2).white(),
            pad_to_width("宗 门", sect_width - 2).white(),
            pad_to_width("当前状态", status_width - 2).white()
        ));
        output.push_str(&format!("{}\n", sep_line.truecolor(180, 100, 200)));
    } else {
        output.push_str("【 仙 界 同 道 】\n");
        output.push_str(&format!("{}\n", header_line));
        output.push_str(&format!(
            "│{}│{}│{}│{}│\n",
            pad_to_width("姓 名", name_width - 2),
            pad_to_width("境 界", realm_width - 2),
            pad_to_width("宗 门", sect_width - 2),
            pad_to_width("当前状态", status_width - 2)
        ));
        output.push_str(&format!("{}\n", sep_line));
    }

    if players.is_empty() {
        let empty_msg = "暂无其他玩家在线";
        if use_color {
            output.push_str(&format!(
                "│{}│{}│{}│{}│\n",
                " ".repeat(name_width - 2),
                " ".repeat(realm_width - 2),
                pad_to_width(empty_msg, sect_width - 2).yellow(),
                " ".repeat(status_width - 2)
            ));
        } else {
            output.push_str(&format!(
                "│{}│{}│{}│{}│\n",
                " ".repeat(name_width - 2),
                " ".repeat(realm_width - 2),
                pad_to_width(empty_msg, sect_width - 2),
                " ".repeat(status_width - 2)
            ));
        }
    } else {
        for (name, realm, sect, status) in &players {
            if use_color {
                let status_str = if *status == "战斗中" {
                    format!("{}", status.red().bold())
                } else if *status == "打坐中" {
                    format!("{}", status.cyan())
                } else {
                    format!("{}", status.green())
                };
                output.push_str(&format!(
                    "│{}│{}│{}│{}│\n",
                    pad_to_width(name, name_width - 2).green().bold(),
                    pad_to_width(realm, realm_width - 2).truecolor(200, 150, 255),
                    pad_to_width(sect, sect_width - 2).yellow(),
                    pad_to_width(&status_str, status_width - 2)
                ));
            } else {
                output.push_str(&format!(
                    "│{}│{}│{}│{}│\n",
                    pad_to_width(name, name_width - 2),
                    pad_to_width(realm, realm_width - 2),
                    pad_to_width(sect, sect_width - 2),
                    pad_to_width(status, status_width - 2)
                ));
            }
        }
    }

    if use_color {
        output.push_str(&format!("{}\n", footer_line.truecolor(180, 100, 200)));
        output.push_str(&format!(
            "{}",
            format!("  ★ 当前共有 {}位 道友在线 ★ ", count)
                .white()
                .bold()
        ));
    } else {
        output.push_str(&format!("{}\n", footer_line));
        output.push_str(&format!(
            "{}",
            format!("  ★ 当前共有 {}位 道友在线 ★ ", count)
        ));
    }

    output
}
