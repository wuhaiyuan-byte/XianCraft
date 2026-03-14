use serde::Deserialize;
use std::collections::{HashMap, HashSet};

#[derive(Deserialize, Debug, Clone)]
pub struct Room {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub exits: HashMap<String, u32>,
    pub npcs: Vec<u32>,
    pub items: Vec<u32>,
    #[serde(skip)]
    pub players: HashSet<u64>,
}

impl Room {
    pub fn get_description(&self) -> &str {
        &self.description
    }

    pub fn get_exits(&self) -> &HashMap<String, u32> {
        &self.exits
    }

    pub fn add_player(&mut self, player_id: u64) {
        self.players.insert(player_id);
    }

    pub fn remove_player(&mut self, player_id: u64) {
        self.players.remove(&player_id);
    }

    pub fn get_npc_ids(&self) -> &Vec<u32> {
        &self.npcs
    }
}
