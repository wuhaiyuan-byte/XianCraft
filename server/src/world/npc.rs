use crate::world::combat::{Combatant};
use crate::world::entity::Entity;
use crate::world::player_state::{PlayerState, BaseAttributes, DerivedStats, Progression};

/// Npc 结构体代表了游戏世界中的非玩家角色。
#[derive(Debug)]
pub struct Npc {
    pub id: usize,
    pub name: String,
    pub state: PlayerState,
    pub room_id: usize, // The room where the NPC is currently located.
}

impl Npc {
    /// 创建一个新的、可供战斗的“强盗”。
    pub fn new_bandit(id: usize, room_id: usize) -> Self {
        Self {
            id,
            name: "强盗".to_string(),
            room_id,
            state: PlayerState {
                base: BaseAttributes {
                    strength: 12,
                    agility: 10,
                    constitution: 11,
                    comprehension: 8,
                },
                derived: DerivedStats {
                    hp: 30,
                    max_hp: 30,
                    mp: 0,
                    max_mp: 0,
                    stamina: 50,
                    max_stamina: 50,
                },
                progression: Progression {
                    level: 2,
                    experience: 10, // Grant 10 XP on defeat
                    potential: 0,
                },
            },
        }
    }
}

// --- Trait Implementations ---

impl Combatant for Npc {
    fn get_id(&self) -> usize {
        self.id
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_state(&self) -> &PlayerState {
        &self.state
    }

    fn get_mut_state(&mut self) -> &mut PlayerState {
        &mut self.state
    }

    fn send_combat_message(&self, message: String) {
        tracing::debug!("[NPC COMBAT LOG - {}]: {}", self.name, message);
    }
}

impl Entity for Npc {
    fn get_id(&self) -> usize {
        self.id
    }

    fn get_entity_type(&self) -> &'static str {
        "Npc"
    }
}

impl AsRef<dyn Combatant> for Npc {
    fn as_ref(&self) -> &(dyn Combatant + 'static) {
        self
    }
}

impl AsMut<dyn Combatant> for Npc {
    fn as_mut(&mut self) -> &mut (dyn Combatant + 'static) {
        self
    }
}
