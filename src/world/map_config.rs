use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct MapConfig {
    pub zone: String,
    pub name: String,
    pub rooms: Vec<RoomConfig>,
    pub npcs: Vec<NpcConfig>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SpawnConfig {
    pub npc: u64,
    pub max: u8,
}

#[derive(Deserialize, Debug)]
pub struct RoomConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub spawns: Vec<SpawnConfig>,
    pub exits: HashMap<String, String>,
    #[serde(default)]
    pub npcs: Vec<u64>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct NpcConfig {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub starting_room: String,
}
