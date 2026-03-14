use crate::combat::CombatState;
use crate::world_model::{Quest, QuestRewards, WorldConfig};
use colored::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// ... (struct definitions remain the same) ...
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Wallet {
    pub crystal: u64,
    pub shell: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BaseStats {
    pub str: u32,
    pub dex: u32,
    pub int: u32,
    pub con: u32,
    pub luk: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlayerQuestStatus {
    pub quest_id: String,
    pub current_step: u32,
    pub is_completed: bool,
    pub kill_counts: HashMap<String, u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Player {
    pub id: u64,
    pub name: String,
    pub nick: Option<String>,
    pub gender: String,
    pub sect: Option<String>,
    pub root_id: String,

    pub stats: BaseStats,
    pub wallet: Wallet,

    pub realm_level: u16,
    pub realm_sub_level: u16,
    pub exp: u64,
    pub potential: u64,
    pub age: u16,
    pub lifespan: u16,

    pub hp: u32,
    pub hp_max: u32,
    pub atk: u32,
    pub qi: u32,
    pub qi_max: u32,
    pub stamina: u32,
    pub stamina_max: u32,

    pub is_resting: bool,
    pub last_input_time: u64,

    pub aliases: HashMap<String, String>,
    pub active_quests: Vec<PlayerQuestStatus>,
    pub completed_quests: HashSet<String>,
    pub quest_counts: HashMap<String, u32>,
    pub inventory: Vec<u32>,
    pub combat_state: Option<CombatState>,
    pub default_skill_id: String,
}

impl Player {
    pub fn new(id: u64, name: String) -> Self {
        Self::new_character(id, name, "凡根".to_string())
    }

    pub fn new_character(id: u64, name: String, root_id: String) -> Self {
        let mut player = Self {
            id,
            name,
            nick: None,
            gender: "未知".to_string(),
            sect: None,
            root_id,
            stats: BaseStats {
                str: 16,
                dex: 16,
                int: 16,
                con: 16,
                luk: 16,
            },
            wallet: Wallet {
                crystal: 0,
                shell: 100,
            },
            realm_level: 1,
            realm_sub_level: 1,
            exp: 0,
            potential: 0,
            age: 16,
            lifespan: 100,
            hp: 0,
            hp_max: 0,
            atk: 0,
            qi: 0,
            qi_max: 0,
            stamina: 100,
            stamina_max: 100,
            is_resting: false,
            last_input_time: 0,
            aliases: HashMap::new(),
            active_quests: Vec::new(),
            completed_quests: HashSet::new(),
            quest_counts: HashMap::new(),
            inventory: Vec::new(),
            combat_state: None,
            default_skill_id: "sword_1".to_string(),
        };
        player.update_vitals();
        player.hp = player.hp_max;
        player.qi = player.qi_max;
        player.stamina = player.stamina_max;
        player
    }

    pub fn accept_quest(&mut self, quest: &Quest) -> bool {
        if self.completed_quests.contains(&quest.id)
            || self.active_quests.iter().any(|q| q.quest_id == quest.id)
        {
            return false;
        }
        self.active_quests.push(PlayerQuestStatus {
            quest_id: quest.id.clone(),
            current_step: 0,
            is_completed: false,
            kill_counts: HashMap::new(),
        });
        true
    }

    pub fn on_kill(&mut self, monster_id: &str, quest_registry: &HashMap<String, Quest>) -> String {
        let mut output = String::new();
        for status in self.active_quests.iter_mut() {
            if let Some(quest) = quest_registry.get(&status.quest_id) {
                if quest.quest_type == "kill" && quest.target_id == monster_id {
                    let count = status
                        .kill_counts
                        .entry(monster_id.to_string())
                        .or_insert(0);
                    *count += 1;

                    let target_count = quest.target_count.unwrap_or(1);
                    output.push_str(&format!(
                        "\n{}",
                        format!("[任务进度] {}: {}/{}", quest.name, count, target_count)
                            .green()
                            .bold()
                    ));

                    if *count >= target_count {
                        status.is_completed = true;
                        output.push_str(&format!(
                            "\n{}",
                            format!("你已达成任务“{}”的目标！", quest.name)
                                .yellow()
                                .bold()
                        ));
                    }
                }
            }
        }
        output
    }

    pub fn update_vitals(&mut self) {
        self.hp_max = self.stats.con * 10 + (self.realm_sub_level as u32 * 30);
        self.atk = 2 * self.realm_sub_level as u32;

        let mut max_qi = (self.stats.con + self.stats.int) * 5;
        if self.root_id == "pseudo" {
            max_qi = (max_qi as f32 * 1.5) as u32;
        }
        self.qi_max = max_qi;
        self.stamina_max = 100;

        if self.hp > self.hp_max {
            self.hp = self.hp_max;
        }
        if self.qi > self.qi_max {
            self.qi = self.qi_max;
        }
        if self.stamina > self.stamina_max {
            self.stamina = self.stamina_max;
        }
    }

    pub fn add_exp(&mut self, amount: u64) -> String {
        self.exp += amount;
        let mut output = String::new();

        while self.realm_sub_level < 9 {
            let need_exp = 100 * (self.realm_sub_level as u64).pow(2);
            if self.exp >= need_exp {
                self.exp -= need_exp;
                self.realm_sub_level += 1;
                self.update_vitals();
                output.push_str(&format!(
                    "\n{}",
                    format!(
                        "【突破】你周身灵气激荡，顺利晋升至[炼气第{}层]！",
                        self.realm_sub_level
                    )
                    .green()
                    .bold()
                ));
            } else {
                break;
            }
        }
        output.push_str(&self.check_promotion());
        output
    }

    pub fn check_promotion(&self) -> String {
        if self.realm_level == 1 && self.realm_sub_level == 9 {
            let need_exp = 100 * 9u64.pow(2);
            if self.exp >= need_exp {
                return format!(
                    "\n{}",
                    "你感到修为已达凡界瓶颈，需寻得筑基丹方可尝试突破筑基期。"
                        .yellow()
                        .bold()
                );
            }
        }
        "".to_string()
    }

    pub fn is_stamina_enough(&self, amount: u32) -> bool {
        self.stamina >= amount
    }

    pub fn consume_stamina(&mut self, amount: u32) -> bool {
        if self.is_stamina_enough(amount) {
            self.stamina -= amount;
            true
        } else {
            false
        }
    }

    pub fn on_heartbeat_recovery(&mut self) {
        let mut recovery = 5 + (self.stats.con / 10);
        if self.is_resting {
            recovery *= 2;
        }
        self.stamina = (self.stamina + recovery).min(self.stamina_max);
    }

    pub fn add_money(&mut self, crystal: u64, shell: u64) {
        self.wallet.crystal += crystal;
        self.wallet.shell += shell;
    }

    pub fn spend_money(&mut self, crystal: u64, shell: u64) -> bool {
        if self.wallet.crystal >= crystal && self.wallet.shell >= shell {
            self.wallet.crystal -= crystal;
            self.wallet.shell -= shell;
            true
        } else {
            false
        }
    }

    pub fn grant_reward(&mut self, rewards: &QuestRewards) -> String {
        let border = "--------------------------------------".yellow().bold();
        let mut output = format!(
            "{}\n{}
{}\n{}\n",
            border,
            "[ 任务圆满完成！ ]".green().bold(),
            border,
            "获得奖励：".white().bold()
        );

        if let Some(shell) = rewards.shell {
            self.wallet.shell += shell;
            output.push_str(&format!(
                "{} {} {}\n",
                "● 灵贝:".cyan(),
                "+".green().bold(),
                shell.to_string().green().bold()
            ));
        }
        if let Some(potential) = rewards.potential {
            self.potential += potential;
            output.push_str(&format!(
                "{} {} {}\n",
                "● 潜能:".cyan(),
                "+".green().bold(),
                potential.to_string().green().bold()
            ));
        }
        if let Some(exp) = rewards.exp {
            let level_up_msg = self.add_exp(exp);
            output.push_str(&format!(
                "{} {} {}{}\n",
                "● 修为:".cyan(),
                "+".green().bold(),
                exp.to_string().green().bold(),
                level_up_msg
            ));
        }
        output.push_str(&border.to_string());
        output
    }

    // Refactored helper function to draw a WoW-style bar
    fn draw_bar(label: &str, current: u32, max: u32, color: Color, width: usize) -> String {
        let fill_width = if max > 0 {
            ((current as f32 / max as f32) * width as f32).round() as usize
        } else {
            0
        };
        let fill_width = fill_width.min(width);

        let fill = "█".repeat(fill_width).color(color);
        let empty = "░".repeat(width - fill_width).truecolor(50, 50, 50); // Dark grey background

        format!(
            "{:<5} {}{} {:>4}/{:<4}",
            label.white(),
            fill,
            empty,
            current,
            max
        )
    }

    // Refactored to use the WoW-style UI
    pub fn get_score_string(&self, _config: &WorldConfig) -> String {
        let realm_name = match self.realm_level {
            1 => format!(
                "炼气{}层",
                match self.realm_sub_level {
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
            _ => "未知".to_string(),
        };

        let border = "══════════════════════════════════════════════".truecolor(100, 100, 100);
        let border_line = format!("╔{}╗", border);
        let end_line = format!("╚{}╝", border);
        let separator = format!("╟{}╢", "─".repeat(46).truecolor(100, 100, 100));

        let mut output = String::new();
        output.push_str(&format!("{}\n", border_line));
        output.push_str(&format!(
            "║ {}{}{}{}{}{}{}{}{}{}\n",
            "姓名：".white(),
            self.name.green().bold(),
            " ".normal(),
            "境界：".white(),
            realm_name.truecolor(163, 53, 238).bold(), // Epic Purple
            " ".normal(),
            "年龄：".white(),
            self.age.to_string().cyan(),
            "/".white(),
            self.lifespan.to_string().cyan()
        ));
        output.push_str(&format!("{}\n", separator));

        // Vitals using the new draw_bar function
        let hp_bar = Self::draw_bar(
            "生命",
            self.hp,
            self.hp_max,
            Color::TrueColor {
                r: 196,
                g: 31,
                b: 59,
            },
            20,
        );
        let qi_bar = Self::draw_bar(
            "真元",
            self.qi,
            self.qi_max,
            Color::TrueColor {
                r: 105,
                g: 204,
                b: 240,
            },
            20,
        );
        let stamina_bar = Self::draw_bar(
            "体力",
            self.stamina,
            self.stamina_max,
            Color::TrueColor {
                r: 255,
                g: 245,
                b: 105,
            },
            20,
        );
        output.push_str(&format!("║  {}  ║\n", hp_bar));
        output.push_str(&format!("║  {}  ║\n", qi_bar));
        output.push_str(&format!("║  {}  ║\n", stamina_bar));
        output.push_str(&format!("{}\n", separator));

        // Stats
        output.push_str(&format!(
            "║  {}{:<4}    {}{:<4}    {}{:<4}  ║\n",
            "力量: ".white(),
            self.stats.str.to_string().truecolor(255, 128, 0), // Orange
            "身法: ".white(),
            self.stats.dex.to_string().green(),
            "悟性: ".white(),
            self.stats.int.to_string().cyan()
        ));
        output.push_str(&format!(
            "║  {}{:<4}    {}{:<4}    {}{:<4}  ║\n",
            "根骨: ".white(),
            self.stats.con.to_string().truecolor(163, 53, 238), // Purple
            "福缘: ".white(),
            self.stats.luk.to_string().yellow(),
            "攻击: ".white(),
            self.atk.to_string().truecolor(196, 31, 59) // Red
        ));
        output.push_str(&format!("{}\n", separator));

        // EXP and Wallet
        output.push_str(&format!(
            "║  {}{:<12} {} {:<10}  ║\n",
            "修为: ".white(),
            self.exp.to_string().green(),
            "潜能: ".white(),
            self.potential.to_string().yellow()
        ));
        output.push_str(&format!(
            "║  {}{:<12} {} {:<10}  ║\n",
            "灵晶: ".white(),
            self.wallet.crystal.to_string().truecolor(163, 53, 238), // Purple
            "灵贝: ".white(),
            self.wallet.shell.to_string().cyan()
        ));

        output.push_str(&format!("{}\n", end_line));
        output
    }

    pub fn display_score(&self, config: &WorldConfig) {
        println!("{}", self.get_score_string(config));
    }
}
