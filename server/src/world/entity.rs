use crate::world::combat::Combatant;
use crate::world::player::Player;
use std::fmt::Debug;

/// Entity 特质是游戏世界中所有“单位”的最高抽象。
/// 无论是玩家、NPC、还是未来可能有的宠物或召唤物，都必须实现这个特质。
pub trait Entity: Debug + Send + Sync + AsMut<dyn Combatant> + AsRef<dyn Combatant> {
    /// 返回实体的唯一ID。
    fn get_id(&self) -> usize;

    /// 返回实体的类型标识，主要用于调试和日志。
    fn get_entity_type(&self) -> &'static str;

    /// 尝试将实体作为 Player 的引用返回。
    /// 这是一个关键的类型转换函数，允许我们从一个通用的 Entity 对象中
    /// 安全地获取 Player 特有的数据（比如 WebSocket sender）。
    fn as_player(&self) -> Option<&Player> {
        None // 默认情况下，一个实体不是玩家。
    }
}
