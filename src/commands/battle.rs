use crate::world::world_player::Player;
use crate::world::world_state::WorldState;
use crate::{combat, ServerMessage};
use colored::*;
use std::time::SystemTime as StdSystemTime;

pub fn handle_attack(
    player: &mut Player,
    target: &str,
    current_room_id: &str,
    world: &WorldState,
    player_id: u64,
    user_name: &str,
) -> Option<ServerMessage> {
    let npcs = world.get_npcs_in_room(current_room_id);
    let npc = npcs
        .iter()
        .find(|n| n.name == target || n.prototype_id.to_string() == target)?;

    let is_monster = if let Some(proto) = world.static_data.npc_prototypes.get(&npc.prototype_id) {
        proto.ai == "monster" || !proto.flags.contains(&"friendly".to_string())
    } else {
        true
    };

    if !is_monster {
        return Some(ServerMessage::Error {
            payload: format!("{} 看起来很友善，你下不了手。", npc.name),
        });
    }

    let tick = StdSystemTime::now()
        .duration_since(StdSystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let default_skill = world
        .static_data
        .skills
        .get("sword_1")
        .or_else(|| world.static_data.skills.values().next())
        .map(|s| s.id.clone())
        .unwrap_or_else(|| "basic_attack".to_string());

    player.combat_state = Some(combat::CombatState::new(
        npc.instance_id.to_string(),
        npc.name.clone(),
        false,
        default_skill.clone(),
        tick,
    ));

    let mut dynamic_data = world.dynamic_data.lock().unwrap();
    if let Some(npc_instance) = dynamic_data.npcs.get_mut(&npc.instance_id) {
        npc_instance.combat_state = Some(combat::CombatState::new(
            player_id.to_string(),
            user_name.to_string(),
            true,
            default_skill,
            tick,
        ));
    }

    let combat_msg = format!("你屏息凝神，锁定了{}，战斗开始！", npc.name.yellow());
    Some(ServerMessage::Description {
        payload: combat_msg,
    })
}

pub fn handle_kill(
    player: &mut Player,
    target: &str,
    current_room_id: &str,
    world: &WorldState,
    player_id: u64,
    user_name: &str,
) -> Option<ServerMessage> {
    let target_npc = {
        let data = world.dynamic_data.lock().unwrap();
        data.npcs
            .values()
            .find(|n| {
                n.current_room == current_room_id
                    && (n.name == target || n.prototype_id.to_string() == target)
            })
            .cloned()
    };

    let npc = target_npc?;

    let is_monster = if let Some(proto) = world.static_data.npc_prototypes.get(&npc.prototype_id) {
        proto.ai == "monster" || !proto.flags.contains(&"friendly".to_string())
    } else {
        true
    };

    if !is_monster {
        return Some(ServerMessage::Error {
            payload: format!("{} 看起来很友善，你下不了手。", npc.name),
        });
    }

    let tick = StdSystemTime::now()
        .duration_since(StdSystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let default_skill = world
        .static_data
        .skills
        .get("sword_1")
        .or_else(|| world.static_data.skills.values().next())
        .map(|s| s.id.clone())
        .unwrap_or_else(|| "basic_attack".to_string());

    let npc_default_skill = {
        let proto = world
            .static_data
            .monsters
            .get(&npc.prototype_id.to_string());
        proto
            .and_then(|m| m.default_skill_id.clone())
            .unwrap_or_else(|| default_skill.clone())
    };

    player.combat_state = Some(combat::CombatState::new(
        npc.instance_id.to_string(),
        npc.name.clone(),
        false,
        default_skill.clone(),
        tick,
    ));

    let mut dynamic_data = world.dynamic_data.lock().unwrap();
    if let Some(npc_instance) = dynamic_data.npcs.get_mut(&npc.instance_id) {
        npc_instance.combat_state = Some(combat::CombatState::new(
            player_id.to_string(),
            user_name.to_string(),
            true,
            npc_default_skill,
            tick,
        ));
    }

    let combat_msg = format!("你屏息凝神，锁定了{}，战斗开始！", npc.name.yellow());
    Some(ServerMessage::Description {
        payload: combat_msg,
    })
}

pub fn handle_cast(
    player: &mut Player,
    skill_id: &str,
    target_name: Option<&str>,
    current_room_id: &str,
    world: &WorldState,
) -> Option<ServerMessage> {
    let skill_tpl = world.static_data.skills.get(skill_id)?.clone();

    if player.qi < skill_tpl.cost_qi as u32 {
        return Some(ServerMessage::Error {
            payload: format!("你的真元不足，需要 {} 点真元。", skill_tpl.cost_qi),
        });
    }

    let attacker_stats = combat::CombatStats {
        hp: player.hp as i32,
        max_hp: player.hp_max as i32,
        attack: player.atk as i32,
        defense: 5,
        level: player.realm_level as i32,
        name: player.name.clone(),
        is_player: true,
        str: player.stats.str as i32,
        dex: player.stats.dex as i32,
        int: player.stats.int as i32,
    };

    if skill_tpl.is_magic && (skill_tpl.base_damage as i32) < 0 {
        let result = combat::resolve_heal(&skill_tpl, &attacker_stats);
        if let combat::CombatResult::Heal { amount, log } = result {
            player.qi -= skill_tpl.cost_qi as u32;
            player.hp = (player.hp + amount as u32).min(player.hp_max);
            return Some(ServerMessage::Description { payload: log });
        }
    }

    let target_npc = if let Some(t) = target_name {
        let data = world.dynamic_data.lock().unwrap();
        data.npcs
            .values()
            .find(|n| {
                n.current_room == current_room_id
                    && (n.name == t || n.prototype_id.to_string() == t)
            })
            .cloned()
    } else {
        None
    };

    let npc = target_npc?;

    let defender_stats = combat::CombatStats {
        hp: npc.hp,
        max_hp: npc.max_hp,
        attack: npc.attack,
        defense: npc.defense,
        level: npc.level,
        name: npc.name.clone(),
        is_player: false,
        str: 10,
        dex: 10,
        int: 10,
    };

    let result = combat::resolve_attack(&attacker_stats, &defender_stats, Some(&skill_tpl));

    player.qi -= skill_tpl.cost_qi as u32;

    match result {
        combat::CombatResult::Hit {
            damage,
            is_crit: _,
            log,
        } => {
            let mut dynamic_data = world.dynamic_data.lock().unwrap();
            if let Some(npc_instance) = dynamic_data.npcs.get_mut(&npc.instance_id) {
                npc_instance.hp -= damage;
                if npc_instance.hp <= 0 {
                    dynamic_data.npcs.remove(&npc.instance_id);
                    return Some(ServerMessage::Description {
                        payload: format!("{}{}", log, format!("\n你击败了{}！", npc.name.yellow())),
                    });
                }
            }
            Some(ServerMessage::Description { payload: log })
        }
        combat::CombatResult::TargetKilled {
            damage: _,
            is_crit: _,
            log,
        } => {
            let mut dynamic_data = world.dynamic_data.lock().unwrap();
            dynamic_data.npcs.remove(&npc.instance_id);
            Some(ServerMessage::Description {
                payload: format!("{}{}", log, format!("\n你击败了{}！", npc.name.yellow())),
            })
        }
        combat::CombatResult::Miss { log } => Some(ServerMessage::Description { payload: log }),
        _ => Some(ServerMessage::Error {
            payload: "技能释放失败。".to_string(),
        }),
    }
}
