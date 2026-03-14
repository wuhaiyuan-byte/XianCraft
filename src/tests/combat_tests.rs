#[cfg(test)]
mod tests {
    use crate::combat::{
        calculate_skill_damage, check_can_cast_skill, get_skill_cost, resolve_attack, CombatResult,
        CombatStats,
    };
    use crate::world_model::{SkillMove, SkillTemplate};

    fn create_test_skill() -> SkillTemplate {
        SkillTemplate {
            id: "test_skill".to_string(),
            name: "测试剑技".to_string(),
            description: "用于测试的剑技".to_string(),
            cost_qi: 20,
            cost_hp: 0,
            base_damage: 30,
            scaling_attr: "str".to_string(),
            scaling_multiplier: 1.5,
            cooldown: 0,
            is_magic: false,
            moves: vec![SkillMove {
                name: "基础攻击".to_string(),
                description: "{attacker}对{defender}发起了攻击".to_string(),
                damage_multiplier: 1.0,
            }],
        }
    }

    fn create_test_skill_low_qi() -> SkillTemplate {
        SkillTemplate {
            id: "high_cost_skill".to_string(),
            name: "高消耗技能".to_string(),
            description: "需要大量真元的技能".to_string(),
            cost_qi: 100,
            cost_hp: 0,
            base_damage: 50,
            scaling_attr: "int".to_string(),
            scaling_multiplier: 2.0,
            cooldown: 0,
            is_magic: true,
            moves: vec![SkillMove {
                name: "强力攻击".to_string(),
                description: "{attacker}对{defender}使出强力一击".to_string(),
                damage_multiplier: 1.5,
            }],
        }
    }

    fn create_attacker() -> CombatStats {
        CombatStats {
            hp: 100,
            max_hp: 100,
            attack: 20,
            defense: 5,
            level: 5,
            name: "测试修士".to_string(),
            is_player: true,
            str: 20,
            dex: 15,
            int: 10,
        }
    }

    fn create_defender() -> CombatStats {
        CombatStats {
            hp: 80,
            max_hp: 80,
            attack: 10,
            defense: 3,
            level: 3,
            name: "翠竹蛇".to_string(),
            is_player: false,
            str: 10,
            dex: 12,
            int: 5,
        }
    }

    #[test]
    fn test_calculate_skill_damage_with_str() {
        let skill = create_test_skill();
        let damage = calculate_skill_damage(&skill, 20, 15, 10);
        assert_eq!(damage, 30 + (20 * 15 / 10) as i32);
    }

    #[test]
    fn test_calculate_skill_damage_with_int() {
        let skill = SkillTemplate {
            scaling_attr: "int".to_string(),
            base_damage: 30,
            scaling_multiplier: 2.0,
            ..create_test_skill()
        };
        let damage = calculate_skill_damage(&skill, 20, 15, 10);
        assert_eq!(damage, 30 + (10 * 20 / 10) as i32);
    }

    #[test]
    fn test_resolve_attack_hit() {
        let attacker = create_attacker();
        let defender = create_defender();

        let result = resolve_attack(&attacker, &defender, None);

        match result {
            CombatResult::Hit {
                damage,
                is_crit,
                log,
            } => {
                assert!(damage > 0);
                assert!(!log.is_empty());
            }
            _ => panic!("Expected Hit result, got {:?}", result),
        }
    }

    #[test]
    fn test_resolve_attack_with_skill() {
        let mut attacker = create_attacker();
        attacker.attack = 5;
        let defender = create_defender();
        let skill = create_test_skill();

        let result = resolve_attack(&attacker, &defender, Some(&skill));

        match result {
            CombatResult::Hit {
                damage,
                is_crit: _,
                log,
            }
            | CombatResult::TargetKilled {
                damage,
                is_crit: _,
                log,
            } => {
                assert!(damage > 0);
                assert!(log.contains("测试剑技"));
            }
            _ => panic!("Expected Hit or TargetKilled result, got {:?}", result),
        }
    }

    #[test]
    fn test_check_can_cast_skill_success() {
        let skill = create_test_skill();
        let can_cast = check_can_cast_skill(&skill, 30);
        assert!(can_cast);
    }

    #[test]
    fn test_check_can_cast_skill_fail() {
        let skill = create_test_skill();
        let can_cast = check_can_cast_skill(&skill, 10);
        assert!(!can_cast);
    }

    #[test]
    fn test_get_skill_cost() {
        let skill = create_test_skill();
        let (qi_cost, hp_cost) = get_skill_cost(&skill);
        assert_eq!(qi_cost, 20);
        assert_eq!(hp_cost, 0);
    }

    #[test]
    fn test_high_cost_skill_insufficient_qi() {
        let skill = create_test_skill_low_qi();
        let can_cast = check_can_cast_skill(&skill, 50);
        assert!(!can_cast);
    }

    #[test]
    fn test_resolve_attack_target_killed() {
        let mut attacker = create_attacker();
        attacker.attack = 100;

        let mut defender = create_defender();
        defender.hp = 10;
        defender.max_hp = 10;

        let result = resolve_attack(&attacker, &defender, None);

        match result {
            CombatResult::TargetKilled {
                damage,
                is_crit: _,
                log,
            } => {
                assert!(damage >= 10);
            }
            _ => panic!("Expected TargetKilled result, got {:?}", result),
        }
    }

    #[test]
    fn test_level_bonus_damage() {
        let mut attacker = create_attacker();
        attacker.level = 10;

        let defender = create_defender();

        let result = resolve_attack(&attacker, &defender, None);

        match result {
            CombatResult::Hit { damage, .. } => {
                assert!(damage > 20);
            }
            _ => panic!("Expected Hit result"),
        }
    }
}
