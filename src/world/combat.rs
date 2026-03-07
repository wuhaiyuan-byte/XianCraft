use crate::world::player::Player;
use crate::world::skill::{Action, Skill};
use crate::world::text::{render, RenderContext};
use rand::seq::SliceRandom;
use rand::Rng;

// For now, we'll assume a default unarmed skill and dodge skill for calculations.
const DEFAULT_ATTACK_SKILL: &str = "unarmed";
const DEFAULT_DODGE_SKILL: &str = "dodge";

/// The main combat engine for the MUD.
pub struct CombatEngine;

/// Represents the outcome of a combat round.
pub struct CombatLog {
    pub messages: Vec<String>,
}

impl CombatEngine {
    /// Executes a full combat round between an attacker and a defender.
    ///
    /// This function orchestrates the entire sequence of a single attack,
    /// from checking cooldowns to dealing damage and generating a log.
    pub fn execute_attack(attacker: &mut Player, defender: &mut Player, skill: &Skill) -> CombatLog {
        let mut messages = Vec::new();

        // 1. Check if the attacker is busy (in a cooldown state).
        if attacker.busy > 0 {
            messages.push(format!("{} is busy and cannot attack.", attacker.name));
            return CombatLog { messages };
        }

        // 2. Select an action based on the attacker's skill level.
        let skill_level = attacker.skills.get(&skill.name).cloned().unwrap_or(0);
        let Some(action) = Self::select_action(skill, skill_level) else {
            messages.push(format!("{} cannot find a suitable action.", attacker.name));
            return CombatLog { messages };
        };

        // Set a cooldown period for the attacker.
        attacker.busy = 3; // Example: 3 game ticks cooldown

        // 3. Check if the attack hits the defender.
        if Self::hit_check(attacker, defender) {
            // 4. Calculate damage based on the action and attacker's stats.
            let damage = Self::calculate_damage(attacker, &action);

            // 5. Apply the damage to the defender.
            defender.take_damage(damage);

            // 6. Generate a combat log with detailed information.
            let context = RenderContext {
                attacker,
                defender,
                body_part: "胸口", // Placeholder
                weapon: "拳头",   // Placeholder
            };
            let message = render(&action.description, &context);
            messages.push(format!("{}, 造成了{}点伤害!", message, damage));

            if !defender.is_alive() {
                messages.push(format!("{}倒下了!", defender.name));
            }
        } else {
            let context = RenderContext {
                attacker,
                defender,
                body_part: "", // Not needed for a miss
                weapon: "拳头", // Placeholder
            };
            let message = render(&action.description, &context); // We might need a different description for misses.
            messages.push(format!("{}, 但是被{}躲过了。", message, defender.name));
        }

        CombatLog { messages }
    }

    /// Selects a random action from a skill based on the character's level.
    pub fn select_action(skill: &Skill, level: u32) -> Option<Action> {
        let mut rng = rand::thread_rng();
        let available_actions: Vec<_> = skill
            .actions
            .iter()
            .filter(|action| action.lvl <= level)
            .collect();
        available_actions.choose(&mut rng).cloned()
    }

    /// Checks if an attack hits based on attacker and defender stats.
    pub fn hit_check(attacker: &Player, defender: &Player) -> bool {
        let mut rng = rand::thread_rng();
        let attack_skill = attacker.skills.get(DEFAULT_ATTACK_SKILL).cloned().unwrap_or(1) as f32;
        let attacker_strength = attacker.attributes.strength as f32;
        let ap = attack_skill * 1.2 + attacker_strength * 2.0 + rng.gen_range(0..10) as f32;

        let dodge_skill = defender.skills.get(DEFAULT_DODGE_SKILL).cloned().unwrap_or(1) as f32;
        let defender_dexterity = defender.attributes.dexterity as f32;
        let dp = dodge_skill * 1.1 + defender_dexterity * 2.0 + rng.gen_range(0..10) as f32;

        ap > dp
    }

    /// Calculates the amount of damage an attacker deals.
    pub fn calculate_damage(attacker: &Player, action: &Action) -> u32 {
        let base_damage = attacker.attributes.strength + action.damage;
        let damage = rand::thread_rng()
            .gen_range((base_damage as f32 * 0.8) as u32..=(base_damage as f32 * 1.2) as u32);
        std::cmp::max(1, damage)
    }
}

// --- Unit Tests ---
#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::skill::{DamageType, SkillClass, SkillType};

    fn create_test_player(
        name: &str,
        strength: u32,
        dexterity: u32,
        unarmed: u32,
        dodge: u32,
    ) -> Player {
        let mut player = Player::new(rand::random(), name.to_string());
        player.attributes.strength = strength;
        player.attributes.dexterity = dexterity;
        player.skills.insert(DEFAULT_ATTACK_SKILL.to_string(), unarmed);
        player.skills.insert(DEFAULT_DODGE_SKILL.to_string(), dodge);
        player
    }

    fn create_test_skill() -> Skill {
        Skill {
            id: 1,
            name: "unarmed".to_string(),
            skill_type: SkillType::Unarmed,
            skill_class: SkillClass::Basic,
            practice_limit: 100,
            actions: vec![
                Action {
                    lvl: 1,
                    damage: 10,
                    force: 1,
                    dodge: 0,
                    parry: 0,
                    damage_type: DamageType::Blunt,
                    description: "$N对著$n的$l挥出了一拳".to_string(),
                },
                Action {
                    lvl: 10,
                    damage: 20,
                    force: 2,
                    dodge: 0,
                    parry: 0,
                    damage_type: DamageType::Blunt,
                    description: "$N朝著$n的$l打出了一记重拳".to_string(),
                },
            ],
            performs: vec![],
        }
    }

    #[test]
    fn test_execute_attack_full_round() {
        let mut attacker = create_test_player("张三", 20, 10, 15, 10);
        let mut defender = create_test_player("李四", 10, 5, 5, 5);
        let skill = create_test_skill();
        let initial_qi = defender.qi;

        let log = CombatEngine::execute_attack(&mut attacker, &mut defender, &skill);

        assert!(!log.messages.is_empty());
        // Check if damage was dealt, implying a successful hit.
        if defender.qi < initial_qi {
            assert!(log.messages[0].contains("伤害"));
        } else {
            assert!(log.messages[0].contains("躲过"));
        }
        assert_eq!(attacker.busy, 3);
    }
}
