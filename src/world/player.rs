use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Player {
    pub id: u64,
    pub name: String,
    pub state: PlayerState,
    pub aliases: HashMap<String, String>,
}

impl Player {
    pub fn new(id: u64, name: String) -> Self {
        Self {
            id,
            name,
            state: PlayerState::new_player_default(),
            aliases: HashMap::new(),
        }
    }
}

/// BaseAttributes 存储了角色的核心基础属性。
/// 这些属性在角色创建时确定，并通过升级或特殊事件来提升。
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BaseAttributes {
    pub strength: i32,      // 力量: 影响物理攻击的伤害
    pub agility: i32,       // 身法: 影响命中率、闪避率和攻击速度
    pub constitution: i32,  // 根骨: 影响生命值和伤害减免
    pub comprehension: i32, // 悟性: 影响学习技能的速度和魔法效果
}

/// DerivedStats 存储了由基础属性计算得出的战斗属性。
/// 这些是角色在战斗中会实时变化的数值。
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DerivedStats {
    pub hp: i32,            // 当前生命值
    pub max_hp: i32,        // 最大生命值
    pub mp: i32,            // 当前法力值
    pub max_mp: i32,        // 最大法力值
    pub stamina: i32,       // 当前耐力值
    pub max_stamina: i32,   // 最大耐力值
}

/// Progression 存储了角色的成长和发展相关的属性。
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Progression {
    pub level: i32,         // 等级
    pub experience: i32,    // 当前经验值
    pub potential: i32,     // 潜能点，用于提升基础属性
}

/// PlayerState 是一个聚合了所有角色状态的顶级结构体。
/// 它代表了一个角色的完整快照，可以被序列化以进行持久化存储。
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlayerState {
    pub base: BaseAttributes,
    pub derived: DerivedStats,
    pub progression: Progression,
}

impl PlayerState {
    /// 创建一个新手玩家的默认状态。
    pub fn new_player_default() -> Self {
        Self {
            base: BaseAttributes {
                strength: 10,
                agility: 10,
                constitution: 10,
                comprehension: 10,
            },
            derived: DerivedStats {
                hp: 50,
                max_hp: 50,
                mp: 20,
                max_mp: 20,
                stamina: 100,
                max_stamina: 100,
            },
            progression: Progression {
                level: 1,
                experience: 0,
                potential: 0,
            },
        }
    }
}
