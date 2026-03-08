use serde::Deserialize;
use std::collections::HashMap;

// From world_config.json
#[derive(Debug, Deserialize, Clone)]
pub struct WorldConfig {
    pub realms: Vec<Realm>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Realm {
    pub name: String,
    pub level_required: u16,
}

// From map files like town.json
#[derive(Debug, Deserialize, Clone)]
pub struct ZoneData {
    pub zone: String,
    pub name: String,
    pub rooms: Vec<Room>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Room {
    pub id: String,
    pub name: String,
    pub description: String,
    pub exits: HashMap<String, String>,
    #[serde(default)]
    pub npcs: Vec<u32>,
    #[serde(default)]
    pub items: Vec<u32>,
}

// Represents the top-level structure of an NPC data file (e.g., npcs.json)
#[derive(Debug, Deserialize, Clone)]
pub struct NpcZoneData {
    pub zone: String,
    pub entities: HashMap<u32, NpcPrototype>,
}

// From npcs.json
#[derive(Debug, Deserialize, Clone)]
pub struct NpcPrototype {
    pub name: String,
    #[serde(default)]
    pub title: String,
    pub description: String,
    pub level: u32,
    pub ai: String,
    #[serde(default)]
    pub flags: Vec<String>,
    pub dialog_id: Option<String>,
    #[serde(default)]
    pub scripts: HashMap<String, String>,
}
