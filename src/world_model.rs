use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// From world_config.json
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WorldConfig {
    pub welcome_message: String,
    pub realms: Vec<Realm>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Realm {
    pub name: String,
    pub level_required: u16,
}

// From map files like town.json
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ZoneData {
    pub zone: String,
    pub name: String,
    pub rooms: Vec<Room>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MultiZoneData {
    pub zones: Vec<ZoneData>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
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
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NpcZoneData {
    pub zone: String,
    pub entities: HashMap<u32, NpcPrototype>,
}

// From npcs.json
#[derive(Debug, Deserialize, Serialize, Clone)]
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

// Represents the top-level structure of an Item data file (e.g., items.json)
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ItemZoneData {
    pub zone: String,
    pub items: HashMap<u32, ItemPrototype>,
}

// From items.json
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ItemPrototype {
    pub name: String,
    #[serde(rename = "type")]
    pub item_type: String, // "type" is a Rust keyword
    pub description: String,
    #[serde(default)]
    pub stats: Option<HashMap<String, i32>>,
    #[serde(default)]
    pub effect: Option<ItemEffect>,
    pub price: ItemPrice,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct ItemEffect {
    #[serde(rename = "type")]
    pub effect_type: String,
    pub target: Option<String>,
    pub value: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ItemPrice {
    pub value: u32,
    pub currency: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QuestStep {
    pub step_id: u32,
    #[serde(rename = "type")]
    pub step_type: String, // "talk" 或 "move"
    pub target_id: String, // NPC ID 或 Room ID
    pub dialog_id: Option<String>,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QuestRewards {
    pub shell: Option<u64>,
    pub potential: Option<u64>,
    pub exp: Option<u64>,
    pub items: Option<Vec<u32>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Quest {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(rename = "type", default)]
    pub quest_type: String, // "kill", "talk", "move", "serial"
    #[serde(default)]
    pub target_id: String,
    pub target_count: Option<u32>,
    #[serde(default)]
    pub steps: Vec<QuestStep>,
    pub rewards: QuestRewards,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QuestRegistry {
    pub quests: HashMap<String, Quest>,
}