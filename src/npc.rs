use crate::world_model::NpcPrototype;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Npc {
    pub instance_id: u64,       // Unique ID for this specific instance
    pub prototype_id: u32,        // ID of the prototype it's based on
    pub name: String,
    pub description: String,
    pub current_room: String,     // The room where the NPC is currently located
    pub combat_target: Option<u64>, // Player ID if in combat
}

impl Npc {
    // Creates a new NPC instance from a prototype and a starting room.
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
        }
    }
}
