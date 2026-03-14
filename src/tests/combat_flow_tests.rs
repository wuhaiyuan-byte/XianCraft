#[cfg(test)]
mod tests {
    use crate::combat::{self, CombatStats};

    #[test]
    fn test_combat_flow() {
        // 模拟玩家属性 (新公式: 10 + str*2 + realm*5)
        let player_stats = CombatStats {
            hp: 190,
            max_hp: 190,
            attack: 47, // 10 + 16*2 + 1*5 = 47
            defense: 5,
            level: 1,
            name: "test_player".to_string(),
            is_player: true,
            str: 16,
            dex: 16,
            int: 16,
        };

        // 模拟怪物属性 (翠竹蛇)
        let npc_stats = CombatStats {
            hp: 50,
            max_hp: 50,
            attack: 12,
            defense: 5,
            level: 2,
            name: "翠竹蛇".to_string(),
            is_player: false,
            str: 10,
            dex: 10,
            int: 10,
        };

        println!("\n=== 战斗流程测试 ===");
        println!(
            "玩家: HP={}, ATK={}, LVL={}",
            player_stats.hp, player_stats.attack, player_stats.level
        );
        println!(
            "怪物: HP={}, ATK={}, LVL={}",
            npc_stats.hp, npc_stats.attack, npc_stats.level
        );

        // 模拟多轮战斗
        let mut player_hp = player_stats.hp;
        let mut npc_hp = npc_stats.hp;

        for round in 1..=10 {
            if npc_hp <= 0 {
                println!("\n回合{} - 怪物死亡! 战斗结束", round);
                break;
            }

            // 玩家攻击怪物
            let result = combat::resolve_attack(&player_stats, &npc_stats, None);
            match result {
                combat::CombatResult::Hit {
                    damage,
                    is_crit,
                    log: _,
                } => {
                    npc_hp -= damage;
                    println!(
                        "回合{} - 玩家攻击: 伤害={}, 暴击={}, 怪物HP={}",
                        round, damage, is_crit, npc_hp
                    );
                }
                combat::CombatResult::Miss { log: _ } => {
                    println!("回合{} - 玩家攻击: 落空!", round);
                }
                _ => {}
            }

            if npc_hp <= 0 {
                println!("回合{} - 怪物死亡! 战斗结束", round);
                break;
            }

            if player_hp <= 0 {
                println!("回合{} - 玩家死亡! 战斗结束", round);
                break;
            }

            // 怪物攻击玩家
            let result = combat::resolve_attack(&npc_stats, &player_stats, None);
            match result {
                combat::CombatResult::Hit {
                    damage,
                    is_crit,
                    log: _,
                } => {
                    player_hp -= damage;
                    println!(
                        "回合{} - 怪物攻击: 伤害={}, 暴击={}, 玩家HP={}",
                        round, damage, is_crit, player_hp
                    );
                }
                combat::CombatResult::Miss { log: _ } => {
                    println!("回合{} - 怪物攻击: 落空!", round);
                }
                _ => {}
            }
        }

        println!("\n=== 战斗测试完成 ===");
        println!("最终结果: 玩家HP={}, 怪物HP={}", player_hp, npc_hp);

        // 验证战斗应该分出胜负
        assert!(npc_hp <= 0 || player_hp <= 0, "战斗应该分出胜负");
    }

    #[test]
    fn test_hit_rate() {
        let player_stats = CombatStats {
            hp: 190,
            max_hp: 190,
            attack: 47,
            defense: 5,
            level: 1,
            name: "玩家".to_string(),
            is_player: true,
            str: 16,
            dex: 16,
            int: 16,
        };

        let npc_stats = CombatStats {
            hp: 50,
            max_hp: 50,
            attack: 12,
            defense: 5,
            level: 2,
            name: "怪物".to_string(),
            is_player: false,
            str: 10,
            dex: 10,
            int: 10,
        };

        println!("\n=== 命中率测试 (100次攻击) ===");

        let mut hits = 0;
        let mut crits = 0;

        for _ in 0..100 {
            let result = combat::resolve_attack(&player_stats, &npc_stats, None);
            match result {
                combat::CombatResult::Hit {
                    damage: _,
                    is_crit,
                    log: _,
                } => {
                    hits += 1;
                    if is_crit {
                        crits += 1;
                    }
                }
                combat::CombatResult::Miss { log: _ } => {}
                _ => {}
            }
        }

        println!("命中次数: {}/100", hits);
        println!("暴击次数: {}/100", crits);
        println!("命中率: {}%", hits);
        println!("暴击率: {}%", crits);

        // 基础命中率应该是 85% (85 + (1-2)*5 = 80~85)
        assert!(hits >= 70, "命中率应该 >= 70%");

        println!("=== 命中率测试完成 ===\n");
    }

    #[test]
    fn test_damage_calculation() {
        let player = CombatStats {
            hp: 190,
            max_hp: 190,
            attack: 47,
            defense: 5,
            level: 1,
            name: "玩家".to_string(),
            is_player: true,
            str: 16,
            dex: 16,
            int: 16,
        };

        let npc = CombatStats {
            hp: 50,
            max_hp: 50,
            attack: 12,
            defense: 5,
            level: 2,
            name: "怪物".to_string(),
            is_player: false,
            str: 10,
            dex: 10,
            int: 10,
        };

        println!("\n=== 伤害计算测试 ===");

        // 基础伤害 = attack + level*2 = 47 + 2 = 49
        // 防御减免 = defense/2 = 2
        // 最终伤害 = 49 - 2 = 47
        let base_dmg = combat::calculate_base_damage(&player);
        let reduction = combat::calculate_defense_reduction(base_dmg, npc.defense);

        println!(
            "基础伤害: {} (attack={} + level*2={})",
            base_dmg,
            player.attack,
            player.level * 2
        );
        println!("防御减免: {} (defense/2 = {})", reduction, npc.defense / 2);
        println!("最终伤害: {}", reduction);

        assert!(reduction > 0, "伤害应该大于0");
        println!("=== 伤害计算测试完成 ===\n");
    }
}
