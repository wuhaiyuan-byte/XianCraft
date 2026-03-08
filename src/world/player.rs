use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use crate::world_model::WorldConfig;

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
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Player {
    pub id: u64,
    pub name: String,
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
}

impl Player {
    pub fn new(id: u64, name: String) -> Self {
        Self::new_character(id, name, "凡根".to_string())
    }

    pub fn new_character(id: u64, name: String, root_id: String) -> Self {
        let mut player = Self {
            id,
            name,
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
        };
        player.update_vitals();
        player.hp = player.hp_max;
        player.qi = player.qi_max;
        player.stamina = player.stamina_max;
        player
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

        if self.hp > self.hp_max { self.hp = self.hp_max; }
        if self.qi > self.qi_max { self.qi = self.qi_max; }
        if self.stamina > self.stamina_max { self.stamina = self.stamina_max; }
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
                output.push_str(&format!("\n\x1b[1;32m【突破】你周身灵气激荡，顺利晋升至[炼气第{}层]！\x1b[0m", self.realm_sub_level));
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
                return "\n\x1b[1;33m你感到修为已达凡界瓶颈，需寻得筑基丹方可尝试突破筑基期。\x1b[0m".to_string();
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

    pub fn grant_reward(&mut self, rewards: &crate::world_model::QuestRewards) -> String {
        let mut output = String::new();
        output.push_str("\x1b[1;33m--------------------------------------\x1b[0m\n");
        output.push_str("\x1b[1;32m[ 任务圆满完成！ ]\x1b[0m\n");
        output.push_str("\x1b[1;33m--------------------------------------\x1b[0m\n");
        output.push_str("\x1b[1;37m获得奖励：\x1b[0m\n");
        
        if let Some(shell) = rewards.shell {
            self.wallet.shell += shell;
            output.push_str(&format!("\x1b[1;36m● 灵贝: \x1b[1;32m+{}\x1b[0m\n", shell));
        }
        if let Some(potential) = rewards.potential {
            self.potential += potential;
            output.push_str(&format!("\x1b[1;36m● 潜能: \x1b[1;32m+{}\x1b[0m\n", potential));
        }
        if let Some(exp) = rewards.exp {
            let level_up_msg = self.add_exp(exp);
            output.push_str(&format!("\x1b[1;36m● 修为: \x1b[1;32m+{}\x1b[0m{}\n", exp, level_up_msg));
        }
        output.push_str("\x1b[1;33m--------------------------------------\x1b[0m");
        output
    }

    fn get_bar(current: u32, max: u32, width: usize) -> String {
        let fill_width = if max > 0 {
            ((current as f32 / max as f32) * width as f32).round() as usize
        } else {
            0
        };
        let fill_width = fill_width.min(width);
        let mut bar = String::from("[");
        for _ in 0..fill_width { bar.push('█'); }
        for _ in fill_width..width { bar.push('░'); }
        bar.push(']');
        bar
    }

    pub fn get_score_string(&self, config: &WorldConfig) -> String {
        let realm_name = match self.realm_level {
            1 => match self.realm_sub_level {
                1 => "炼气一层",
                2 => "炼气二层",
                3 => "炼气三层",
                4 => "炼气四层",
                5 => "炼气五层",
                6 => "炼气六层",
                7 => "炼气七层",
                8 => "炼气八层",
                9 => "炼气九层",
                _ => "炼气期",
            },
            _ => "未知",
        };

        let hp_bar = Self::get_bar(self.hp, self.hp_max, 10);
        let qi_bar = Self::get_bar(self.qi, self.qi_max, 10);
        let stamina_bar = Self::get_bar(self.stamina, self.stamina_max, 10);

        let mut output = String::new();
        output.push_str("\x1b[1;33m--------------------------------------\x1b[0m\n");
        output.push_str(&format!("\x1b[1;37m姓名：\x1b[1;32m{:<10}\x1b[1;37m  境界：\x1b[1;35m{}\x1b[0m\n", self.name, realm_name));
        output.push_str(&format!("\x1b[1;37m灵根：\x1b[1;34m{:<10}\x1b[1;37m  年龄：\x1b[1;36m{}/{}\x1b[0m\n", self.root_id, self.age, self.lifespan));
        output.push_str("\x1b[1;33m--------------------------------------\x1b[0m\n");
        output.push_str(&format!("\x1b[1;37m生命: \x1b[1;31m{} \x1b[0m{:>4}/{:<4}\n", hp_bar, self.hp, self.hp_max));
        output.push_str(&format!("\x1b[1;37m真元: \x1b[1;34m{} \x1b[0m{:>4}/{:<4}\n", qi_bar, self.qi, self.qi_max));
        output.push_str(&format!("\x1b[1;37m体力: \x1b[1;32m{} \x1b[0m{:>4}/{:<4}\n", stamina_bar, self.stamina, self.stamina_max));
        output.push_str("\x1b[1;33m--------------------------------------\x1b[0m\n");
        output.push_str(&format!("\x1b[1;37m力量(STR): \x1b[1;31m{:<4}\x1b[1;37m    身法(DEX): \x1b[1;32m{:<4}\x1b[0m\n", self.stats.str, self.stats.dex));
        output.push_str(&format!("\x1b[1;37m悟性(INT): \x1b[1;36m{:<4}\x1b[1;37m    根骨(CON): \x1b[1;35m{:<4}\x1b[0m\n", self.stats.int, self.stats.con));
        output.push_str(&format!("\x1b[1;37m福缘(LUK): \x1b[1;33m{:<4}\x1b[1;37m    攻击(ATK): \x1b[1;31m{:<4}\x1b[0m\n", self.stats.luk, self.atk));
        output.push_str("\x1b[1;33m--------------------------------------\x1b[0m\n");
        output.push_str(&format!("\x1b[1;37m修为: \x1b[1;32m{:<10}\x1b[1;37m  潜能: \x1b[1;33m{}\x1b[0m\n", self.exp, self.potential));
        output.push_str("\x1b[1;33m--------------------------------------\x1b[0m\n");
        output.push_str(&format!("\x1b[1;37m储物空间资产：\x1b[0m\n"));
        output.push_str(&format!("\x1b[1;37m灵晶: \x1b[1;35m{:<10}\x1b[1;37m  灵贝: \x1b[1;36m{}\x1b[0m\n", self.wallet.crystal, self.wallet.shell));
        output.push_str("\x1b[1;33m--------------------------------------\x1b[0m");
        output
    }

    pub fn display_score(&self, config: &WorldConfig) {
        println!("{}", self.get_score_string(config));
    }
}