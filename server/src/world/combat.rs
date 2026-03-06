use crate::world::player_state::PlayerState;
use rand::seq::SliceRandom;
use rand::Rng;

//---------- Combatant Trait ----------//

pub trait Combatant {
    fn get_id(&self) -> usize;
    fn get_name(&self) -> &str;
    fn get_state(&self) -> &PlayerState;
    fn get_mut_state(&mut self) -> &mut PlayerState;
    fn send_combat_message(&self, message: String);
}

//---------- Attack Flavor & Descriptions ----------//

struct AttackFlavor {
    action: &'static str,
    hit_on_d: &'static str,
    miss_on_d: &'static str,
    hit_on_self: &'static str,
    miss_on_self: &'static str,
}

// Using standard ASCII punctuation to avoid compiler errors.
const UNARMED_ATTACKS: &[AttackFlavor] = &[
    AttackFlavor {
        action: "黑虎掏心",
        hit_on_d: "正中$d的胸口, $d闷哼一声!",
        miss_on_d: "被$d机敏地闪了开去.",
        hit_on_self: "正中你的胸口, 你感觉一阵气血翻涌!",
        miss_on_self: "被你机敏地闪开了.",
    },
    AttackFlavor {
        action: "猛虎下山",
        hit_on_d: "狠狠地砸在了$d的肩膀上!",
        miss_on_d: "被$d一个懒驴打滚, 险之又险地避开了.",
        hit_on_self: "你躲闪不及, 肩膀传来一阵剧痛!",
        miss_on_self: "你一个懒驴打滚, 险之又险地避开了.",
    },
    AttackFlavor {
        action: "直捣黄龙",
        hit_on_d: "正中$d的面门, 打得$d眼冒金星!",
        miss_on_d: "被$d轻轻一侧头, 躲了过去.",
        hit_on_self: "拳头结结实实地打在了你的面门上, 你顿时眼冒金星!",
        miss_on_self: "你轻轻一侧头, 躲了过去.",
    },
];

fn get_unarmed_attack_flavor() -> &'static AttackFlavor {
    UNARMED_ATTACKS.choose(&mut rand::thread_rng()).unwrap()
}

//---------- Damage Reaction Descriptions ----------//

struct DamageReaction {
    desc_for_a: &'static str,
    desc_for_d: &'static str,
    class: &'static str,
}

const DAMAGE_REACTIONS: &[DamageReaction] = &[
    DamageReaction {
        desc_for_a: "( $d看起来似乎没什么大碍. )",
        desc_for_d: "( 你看起来似乎没什么大碍. )",
        class: "reaction-light",
    },
    DamageReaction {
        desc_for_a: "( $d闷哼了一声, 脸色有些发白. )",
        desc_for_d: "( 你闷哼了一声, 脸色有些发白. )",
        class: "reaction-medium",
    },
    DamageReaction {
        desc_for_a: "( $d受到了不轻的创伤, 脚步变得有些不稳. )",
        desc_for_d: "( 你受到了不轻的创伤, 脚步变得有些不稳. )",
        class: "reaction-heavy",
    },
];

fn get_damage_reaction(current_hp: i32, max_hp: i32) -> &'static DamageReaction {
    let hp_percent = current_hp as f32 / max_hp as f32;
    if hp_percent > 0.7 { &DAMAGE_REACTIONS[0] }
    else if hp_percent > 0.3 { &DAMAGE_REACTIONS[1] }
    else { &DAMAGE_REACTIONS[2] }
}


//---------- Core Combat Logic ----------//

#[derive(serde::Serialize, Clone, Debug)]
pub struct AttackResult {
    pub attacker_id: usize,
    pub defender_id: usize,
    pub damage_dealt: i32,
    pub is_hit: bool,
    pub defender_is_dead: bool,
}

pub fn resolve_attack_round(attacker: &mut dyn Combatant, defender: &mut dyn Combatant) -> AttackResult {
    let attacker_name = attacker.get_name().to_string();
    let defender_name = defender.get_name().to_string();
    let flavor = get_unarmed_attack_flavor();

    let mut result = AttackResult {
        attacker_id: attacker.get_id(),
        defender_id: defender.get_id(),
        damage_dealt: 0,
        is_hit: false,
        defender_is_dead: false,
    };

    let hit_chance = calculate_hit_chance(attacker, defender);
    if rand::thread_rng().gen_bool(hit_chance as f64) {
        // --- HIT ---
        result.is_hit = true;
        let damage = calculate_physical_damage(attacker);
        result.damage_dealt = damage;

        let is_dead = apply_damage(defender, damage);
        result.defender_is_dead = is_dead;
        
        if is_dead {
            let attacker_msg = format!(
                "<span class=\"combat-base\">你使出一式 </span><span class=\"combat-skill\">「{}」</span><span class=\"combat-base\">, {} 造成了</span><span class=\"combat-damage\">{}</span><span class=\"combat-base\">点伤害.</span>",
                flavor.action, 
                flavor.hit_on_d.replace("$d", &defender_name),
                damage
            );
            attacker.send_combat_message(attacker_msg);

            let defender_msg = format!(
                "<span class=\"combat-base\">{}对你使出一式 </span><span class=\"combat-skill\">「{}」</span><span class=\"combat-base\">, {} 你受到了</span><span class=\"combat-damage\">{}</span><span class=\"combat-base\">点伤害.</span>",
                attacker_name, 
                flavor.action,
                flavor.hit_on_self,
                damage
            );
            defender.send_combat_message(defender_msg);

            attacker.send_combat_message(format!("<span class=\"reaction-crit\">{}惨叫一声, 倒在了血泊之中.</span>", defender_name));
            defender.send_combat_message("<span class=\"reaction-crit\">你眼前一黑, 失去了知觉...</span>".to_string());

            let xp_gain = 50;
            if attacker.get_state().progression.level < 100 { // Cap XP gain
                attacker.get_mut_state().progression.experience += xp_gain;
                attacker.send_combat_message(format!("<span class=\"combat-xp\">你获得了 {} 点经验.</span>", xp_gain));
            }

        } else {
            let def_state = defender.get_state();
            let reaction = get_damage_reaction(def_state.derived.hp, def_state.derived.max_hp);

            let attacker_msg = format!(
                "<span class=\"combat-base\">你使出一式 </span><span class=\"combat-skill\">「{}」</span><span class=\"combat-base\">, {} 造成了</span><span class=\"combat-damage\">{}</span><span class=\"combat-base\">点伤害.</span><br/><span class=\"{}\">{}</span>",
                flavor.action, 
                flavor.hit_on_d.replace("$d", &defender_name),
                damage,
                reaction.class,
                reaction.desc_for_a.replace("$d", &defender_name)
            );
            attacker.send_combat_message(attacker_msg);

            let defender_msg = format!(
                "<span class=\"combat-base\">{}对你使出一式 </span><span class=\"combat-skill\">「{}」</span><span class=\"combat-base\">, {} 你受到了</span><span class=\"combat-damage\">{}</span><span class=\"combat-base\">点伤害.</span><br/><span class=\"{}\">{}</span>",
                attacker_name, 
                flavor.action,
                flavor.hit_on_self,
                damage,
                reaction.class,
                reaction.desc_for_d
            );
            defender.send_combat_message(defender_msg);
        }
    } else {
        // --- MISS ---
        result.is_hit = false;
        let attacker_msg = format!(
            "<span class=\"combat-base\">你使出一式 </span><span class=\"combat-skill\">「{}」</span><span class=\"combat-base\">, 但<span class=\"combat-miss\">{}</span></span>", 
            flavor.action, 
            flavor.miss_on_d.replace("$d", &defender_name)
        );
        attacker.send_combat_message(attacker_msg);

        let defender_msg = format!(
            "<span class=\"combat-base\">{}对你使出一式 </span><span class=\"combat-skill\">「{}」</span><span class=\"combat-base\">, 但<span class=\"combat-miss\">{}</span></span>", 
            attacker_name, 
            flavor.action, 
            flavor.miss_on_self
        );
        defender.send_combat_message(defender_msg);
    }

    result
}

//---------- Helper Functions ----------//

fn calculate_physical_damage(attacker: &dyn Combatant) -> i32 {
    let state = attacker.get_state();
    let base_damage = state.base.strength;
    // Add a bit more variance to damage
    let damage = rand::thread_rng().gen_range((base_damage as f32 * 0.9) as i32..(base_damage as f32 * 1.3) as i32);
    std::cmp::max(1, damage)
}

fn apply_damage(defender: &mut dyn Combatant, amount: i32) -> bool {
    let state = defender.get_mut_state();
    state.derived.hp -= amount;
    if state.derived.hp < 0 {
        state.derived.hp = 0;
    }
    state.derived.hp == 0
}

fn calculate_hit_chance(attacker: &dyn Combatant, defender: &dyn Combatant) -> f32 {
    let attacker_agility = attacker.get_state().base.agility as f32;
    let defender_agility = defender.get_state().base.agility as f32;
    // Agility has a stronger effect on hit chance
    let chance = 0.85 + (attacker_agility - defender_agility) * 0.025;
    chance.clamp(0.1, 0.99)
}
