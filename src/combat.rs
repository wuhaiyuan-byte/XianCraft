use crate::world_model::SkillTemplate;
use colored::*;
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CombatResult {
    Hit {
        damage: i32,
        is_crit: bool,
        log: String,
    },
    Miss {
        log: String,
    },
    TargetKilled {
        damage: i32,
        is_crit: bool,
        log: String,
    },
    InsufficientQi {
        log: String,
    },
    Heal {
        amount: i32,
        log: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatState {
    pub target_id: String,
    pub target_name: String,
    pub target_is_player: bool,
    pub current_skill_id: String,
    pub combo_index: usize,
    pub last_attack_tick: u64,
    pub is_in_combat: bool,
}

impl CombatState {
    pub fn new(
        target_id: String,
        target_name: String,
        target_is_player: bool,
        skill_id: String,
        tick: u64,
    ) -> Self {
        Self {
            target_id,
            target_name,
            target_is_player,
            current_skill_id: skill_id,
            combo_index: 0,
            last_attack_tick: tick,
            is_in_combat: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CombatStats {
    pub hp: i32,
    pub max_hp: i32,
    pub attack: i32,
    pub defense: i32,
    pub level: i32,
    pub name: String,
    pub is_player: bool,
    pub str: i32,
    pub dex: i32,
    pub int: i32,
}

pub fn calculate_base_damage(attacker_stats: &CombatStats) -> i32 {
    let base = attacker_stats.attack;
    let level_bonus = attacker_stats.level * 2;
    base + level_bonus
}

pub fn calculate_defense_reduction(damage: i32, defender_defense: i32) -> i32 {
    let reduction = defender_defense / 2;
    (damage - reduction).max(1)
}

pub fn resolve_attack(
    attacker_stats: &CombatStats,
    defender_stats: &CombatStats,
    skill_opt: Option<&SkillTemplate>,
) -> CombatResult {
    let mut rng = rand::thread_rng();

    // Base hit rate is 80%, level difference affects it
    let level_diff = attacker_stats.level - defender_stats.level;
    let hit_chance = (80 + level_diff * 5).clamp(50, 95);

    if rng.gen_range(0..100) > hit_chance {
        return CombatResult::Miss {
            log: format!(
                "你{}对{}的攻击落空了！",
                attacker_stats.name.yellow(),
                defender_stats.name.cyan()
            ),
        };
    }

    let mut base_damage = calculate_base_damage(attacker_stats);

    if let Some(skill) = skill_opt {
        base_damage = calculate_skill_damage(
            skill,
            attacker_stats.str,
            attacker_stats.dex,
            attacker_stats.int,
        );
    }

    let crit_chance = 10 + (attacker_stats.level / 5);
    let is_crit = rng.gen_range(0..100) < crit_chance;

    if is_crit {
        base_damage = (base_damage as f32 * 1.5) as i32;
    }

    let final_damage = calculate_defense_reduction(base_damage, defender_stats.defense);
    let will_kill = defender_stats.hp - final_damage <= 0;

    let log = if is_crit {
        format!(
            "你{}运转功法，对{}使出{}，造成了{}点{}！",
            attacker_stats.name.yellow(),
            defender_stats.name.cyan(),
            skill_opt.map(|s| s.name.as_str()).unwrap_or("普通攻击"),
            final_damage.to_string().red().bold(),
            "暴击！".red().bold()
        )
    } else {
        format!(
            "你{}对{}使出{}，造成了{}点伤害。",
            attacker_stats.name.yellow(),
            defender_stats.name.cyan(),
            skill_opt.map(|s| s.name.as_str()).unwrap_or("普通攻击"),
            final_damage.to_string().red()
        )
    };

    if will_kill {
        CombatResult::TargetKilled {
            damage: final_damage,
            is_crit,
            log,
        }
    } else {
        CombatResult::Hit {
            damage: final_damage,
            is_crit,
            log,
        }
    }
}

pub fn calculate_skill_damage(skill: &SkillTemplate, str: i32, dex: i32, int: i32) -> i32 {
    let scaling_attr_value = match skill.scaling_attr.as_str() {
        "str" => str as f32,
        "dex" => dex as f32,
        "int" => int as f32,
        "con" => 16.0,
        _ => 16.0,
    };

    let scaling_damage = (scaling_attr_value * skill.scaling_multiplier) as i32;
    (skill.base_damage as i32) + scaling_damage
}

pub fn resolve_heal(skill: &SkillTemplate, healer_stats: &CombatStats) -> CombatResult {
    let heal_amount =
        calculate_skill_damage(skill, healer_stats.str, healer_stats.dex, healer_stats.int).abs();
    let actual_heal = (heal_amount as f32 * 0.5) as i32;
    let capped_heal = actual_heal.min(healer_stats.max_hp - healer_stats.hp);

    CombatResult::Heal {
        amount: capped_heal,
        log: format!(
            "你{}运转灵气，{}的治疗波笼罩了你自身，恢复了{}点生命！",
            healer_stats.name.yellow(),
            skill.name.cyan(),
            capped_heal.to_string().green().bold()
        ),
    }
}

pub fn check_can_cast_skill(skill: &SkillTemplate, current_qi: i32) -> bool {
    current_qi >= skill.cost_qi as i32
}

pub fn get_skill_cost(skill: &SkillTemplate) -> (i32, i32) {
    (skill.cost_qi as i32, skill.cost_hp as i32)
}

pub fn process_combat_move(
    attacker_stats: &CombatStats,
    defender_stats: &CombatStats,
    skill: &SkillTemplate,
    combo_index: usize,
) -> (i32, String, bool) {
    let mut rng = rand::thread_rng();

    let moves = &skill.moves;
    if moves.is_empty() {
        let base_damage = calculate_base_damage(attacker_stats);
        let final_damage = calculate_defense_reduction(base_damage, defender_stats.defense);
        let is_crit = rng.gen_range(0..100) < 15;
        let damage = if is_crit {
            (final_damage as f32 * 1.5) as i32
        } else {
            final_damage
        };
        let log = format!(
            "{} 对 {} 发起了攻击，造成了 {} 点伤害",
            attacker_stats.name.yellow(),
            defender_stats.name.cyan(),
            damage.to_string().red()
        );
        return (damage, log, is_crit);
    }

    let move_idx = combo_index % moves.len();
    let mve = &moves[move_idx];

    let base_damage = calculate_base_damage(attacker_stats);
    let scaled_skill_damage = calculate_skill_damage(
        skill,
        attacker_stats.str,
        attacker_stats.dex,
        attacker_stats.int,
    );
    let damage_with_multiplier = (scaled_skill_damage as f32 * mve.damage_multiplier) as i32;
    let final_damage = calculate_defense_reduction(damage_with_multiplier, defender_stats.defense);

    let is_crit = rng.gen_range(0..100) < 15;
    let damage = if is_crit {
        (final_damage as f32 * 1.5) as i32
    } else {
        final_damage
    };

    let mut description = mve.description.clone();
    description = description.replace("{attacker}", &format!("{}", attacker_stats.name.yellow()));
    description = description.replace("{defender}", &format!("{}", defender_stats.name.cyan()));

    if is_crit {
        description.push_str(" ");
        description.push_str(&format!("{}暴击！", "【暴击】".red().bold()));
    }

    let damage_text = format!("[伤害: {}]", damage.to_string().red().bold());
    let log = format!("{} {}", damage_text, description);

    (damage, log, is_crit)
}

pub fn get_health_state_description(
    current_hp: i32,
    max_hp: i32,
    health_states: &[crate::world_model::HealthStateTemplate],
) -> String {
    if health_states.is_empty() {
        return "状态良好".to_string();
    }

    let hp_ratio = current_hp as f32 / max_hp as f32;

    for state in health_states.iter() {
        if hp_ratio >= state.hp_threshold {
            return state.description.clone();
        }
    }

    health_states
        .last()
        .map(|s| s.description.clone())
        .unwrap_or_else(|| "重伤濒死".to_string())
}

pub fn get_default_skill_for_player() -> String {
    "sword_1".to_string()
}
