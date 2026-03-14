use crate::world_model::NpcPrototype;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Npc {
    pub instance_id: u64,
    pub prototype_id: u32,
    pub name: String,
    pub description: String,
    pub current_room: String,
    pub combat_target: Option<u64>,
    pub dialog_id: Option<String>,
    pub display_prefix: String,

    // Combat stats
    pub hp: i32,
    pub max_hp: i32,
    pub attack: i32,
    pub defense: i32,
    pub level: i32,
}

impl Npc {
    pub fn from_prototype(
        instance_id: u64,
        prototype_id: u32,
        prototype: &NpcPrototype,
        room_id: String,
    ) -> Self {
        Self {
            instance_id,
            prototype_id,
            name: prototype.name.clone(),
            description: prototype.description.clone(),
            current_room: room_id,
            combat_target: None,
            dialog_id: prototype.dialog_id.clone(),
            display_prefix: if prototype.display_prefix.is_empty() {
                "NPC".to_string()
            } else {
                prototype.display_prefix.clone()
            },
            hp: 0,
            max_hp: 0,
            attack: 0,
            defense: 0,
            level: prototype.level as i32,
        }
    }

    pub fn init_combat_stats(&mut self, max_hp: i32, attack: i32, defense: i32) {
        self.max_hp = max_hp;
        self.hp = max_hp;
        self.attack = attack;
        self.defense = defense;
    }
}
