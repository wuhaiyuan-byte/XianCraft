use serde::{Deserialize, Serialize};

/// 技能类型
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum SkillType {
    Unarmed,   // 拳脚
    Sword,     // 剑法
    Blade,     // 刀法
    Staff,     // 杖法
    Whip,      // 鞭法
    Throwing,  // 暗器
    Poison,    // 毒技
    Internal,  // 内功
    Lightness, // 轻功
    Knowledge, // 知识
}

/// 技能等级
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum SkillClass {
    Basic,     // 基础
    Advanced,  // 高级
    Special,   // 特殊
    Forbidden, // 禁术
}

/// 伤害类型
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum DamageType {
    Blunt,    // 钝器
    Slash,    // 割伤
    Pierce,   // 刺伤
    Internal, // 内伤
    Poison,   // 毒伤
}

/// 技能在不同等级下的效果
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Action {
    pub lvl: u32,                // 技能等级
    pub damage: u32,             // 伤害
    pub force: u32,              // 力道
    pub dodge: u32,              // 闪避
    pub parry: u32,              // 招架
    pub damage_type: DamageType, // 伤害类型
    pub description: String,     // 描述
}

/// 技能的特殊招式 (Perform)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Perform {
    // 您没有指定 Perform 的字段，我暂时将它留空。
    // 常见字段可以包括: id, name, description, cost 等。
}

/// 核心技能结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Skill {
    pub id: u64,
    pub name: String,
    pub skill_type: SkillType,
    pub skill_class: SkillClass,
    pub practice_limit: u32, // 练习上限
    pub actions: Vec<Action>,
    pub performs: Vec<Perform>,
}
