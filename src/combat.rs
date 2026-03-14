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

// ============================================================================
// 第一步：提取公共计算辅助函数
// ============================================================================

fn roll_hit_chance(attacker_level: i32, defender_level: i32) -> bool {
    let mut rng = rand::thread_rng();
    let level_diff = attacker_level - defender_level;
    // 基础命中率85%，最低70%，最高95%
    let hit_chance = (85 + level_diff * 5).clamp(70, 95);
    rng.gen_range(0..100) < hit_chance
}

fn roll_crit_chance(base_level: i32) -> bool {
    let mut rng = rand::thread_rng();
    let crit_chance = 10 + (base_level / 5);
    rng.gen_range(0..100) < crit_chance
}

fn calc_defense_mitigation(raw_damage: i32, defense: i32) -> i32 {
    let reduction = defense / 2;
    (raw_damage - reduction).max(1)
}

fn apply_crit_damage(damage: i32, is_crit: bool) -> i32 {
    if is_crit {
        (damage as f32 * 1.5) as i32
    } else {
        damage
    }
}

fn format_damage(damage: i32, is_crit: bool) -> String {
    if is_crit {
        format!("{}", damage.to_string().red().bold())
    } else {
        format!("{}", damage.to_string().red())
    }
}

// ============================================================================
// 公共导出函数
// ============================================================================

pub fn calculate_base_damage(attacker_stats: &CombatStats) -> i32 {
    let base = attacker_stats.attack;
    let level_bonus = attacker_stats.level * 2;
    base + level_bonus
}

pub fn calculate_defense_reduction(damage: i32, defender_defense: i32) -> i32 {
    calc_defense_mitigation(damage, defender_defense)
}

pub fn resolve_attack(
    attacker_stats: &CombatStats,
    defender_stats: &CombatStats,
    skill_opt: Option<&SkillTemplate>,
) -> CombatResult {
    if !roll_hit_chance(attacker_stats.level, defender_stats.level) {
        return CombatResult::Miss {
            log: format!(
                "{}身形如电，{}但见招式落空！",
                attacker_stats.name.yellow(),
                defender_stats.name.cyan()
            ),
        };
    }

    let base_damage = if let Some(skill) = skill_opt {
        calculate_skill_damage(
            skill,
            attacker_stats.str,
            attacker_stats.dex,
            attacker_stats.int,
        )
    } else {
        calculate_base_damage(attacker_stats)
    };

    let is_crit = roll_crit_chance(attacker_stats.level);
    let damage_after_crit = apply_crit_damage(base_damage, is_crit);
    let final_damage = calc_defense_mitigation(damage_after_crit, defender_stats.defense);
    let will_kill = defender_stats.hp - final_damage <= 0;

    let skill_name = skill_opt.map(|s| s.name.as_str()).unwrap_or("普通攻击");
    let damage_str = format_damage(final_damage, is_crit);

    let log = if is_crit {
        if skill_opt.is_some() {
            format!(
                "{}周身灵气激荡，施展出【{}】，{}身受重创！\n→ [伤害: {}] {}!",
                attacker_stats.name.yellow(),
                skill_name,
                defender_stats.name.cyan(),
                damage_str,
                "暴击".red().bold()
            )
        } else {
            format!(
                "{}身形如电，一记重击狠狠轰在{}身上！\n→ [伤害: {}] {}!",
                attacker_stats.name.yellow(),
                defender_stats.name.cyan(),
                damage_str,
                "暴击".red().bold()
            )
        }
    } else {
        if skill_opt.is_some() {
            format!(
                "{}催动真元，施展出【{}】，正中{}！\n→ [伤害: {}]",
                attacker_stats.name.yellow(),
                skill_name,
                defender_stats.name.cyan(),
                damage_str
            )
        } else {
            format!(
                "{}运转功法，对{}发起了攻击。\n→ [伤害: {}]",
                attacker_stats.name.yellow(),
                defender_stats.name.cyan(),
                damage_str
            )
        }
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
            "{}闭目凝神，运转【{}】功法，\n淡蓝色灵气笼罩全身，恢复了{}点生命！",
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

// ============================================================================
// 第二步：重构 process_combat_move (自动连招逻辑)
// ============================================================================

pub fn process_combat_move(
    attacker_stats: &CombatStats,
    defender_stats: &CombatStats,
    skill: &SkillTemplate,
    combo_index: usize,
) -> (i32, String, bool) {
    let moves = &skill.moves;

    // 无连招时使用普通攻击逻辑
    if moves.is_empty() {
        let base_damage = calculate_base_damage(attacker_stats);
        let final_damage = calc_defense_mitigation(base_damage, defender_stats.defense);
        let is_crit = roll_crit_chance(attacker_stats.level);
        let damage = apply_crit_damage(final_damage, is_crit);

        let log = if is_crit {
            format!(
                "{}身形如电，一记重击狠狠轰在{}身上！\n→ [伤害: {}] {}!",
                attacker_stats.name.yellow(),
                defender_stats.name.cyan(),
                format_damage(damage, true),
                "暴击".red().bold()
            )
        } else {
            format!(
                "{}运转功法，对{}发起了攻击。\n→ [伤害: {}]",
                attacker_stats.name.yellow(),
                defender_stats.name.cyan(),
                format_damage(damage, false)
            )
        };
        return (damage, log, is_crit);
    }

    // 连招逻辑：基础攻击力 + 技能伤害 → 乘以招式倍率 → 减去防御
    let move_idx = combo_index % moves.len();
    let mve = &moves[move_idx];

    let base_atk_damage = calculate_base_damage(attacker_stats);
    let skill_damage = calculate_skill_damage(
        skill,
        attacker_stats.str,
        attacker_stats.dex,
        attacker_stats.int,
    );
    let combined_damage = base_atk_damage + skill_damage;
    let damage_with_multiplier = (combined_damage as f32 * mve.damage_multiplier) as i32;
    let final_damage_raw = calc_defense_mitigation(damage_with_multiplier, defender_stats.defense);

    let is_crit = roll_crit_chance(attacker_stats.level);
    let final_damage = apply_crit_damage(final_damage_raw, is_crit);

    // 渲染招式描述
    let mut description = mve.description.clone();
    description = description.replace("{attacker}", &format!("{}", attacker_stats.name.yellow()));
    description = description.replace("{defender}", &format!("{}", defender_stats.name.cyan()));

    // 构建日志：招式名 + 描述 + 伤害 + 暴击提示
    let crit_text = if is_crit {
        format!(" {}", "【暴击】".red().bold())
    } else {
        String::new()
    };

    let log = format!(
        "【{}】{}\n→ [伤害: {}]{}",
        mve.name.cyan(),
        description,
        format_damage(final_damage, is_crit),
        crit_text
    );

    (final_damage, log, is_crit)
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
