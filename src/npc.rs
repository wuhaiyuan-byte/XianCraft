use crate::world::world::World;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Npc {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub current_room: String,
    pub combat_target: Option<u64>,
}

impl Npc {
    pub fn new(id: u64, name: &str, description: &str, starting_room: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            description: description.to_string(),
            current_room: starting_room.to_string(),
            combat_target: None,
        }
    }

    pub async fn tick(&mut self, world: Arc<World>) {
        if let Some(_target_id) = self.combat_target {
            // Attack back
            // println!("{} attacks player {}", self.name, target_id);
        } else {
            // Check for players in the same room
            if let Some(room) = world.get_room(&self.current_room) {
                let players_in_room = room.get_player_ids();
                if !players_in_room.is_empty() {
                    // 50% chance to attack a random player
                    if rand::random::<f32>() < 0.5 {
                        if let Some(target_id) = players_in_room.get(rand::random::<usize>() % players_in_room.len()) {
                            self.combat_target = Some(*target_id);
                            // println!("{} attacks player {}", self.name, target_id);
                        }
                    }
                }
            }
        }
    }
}
