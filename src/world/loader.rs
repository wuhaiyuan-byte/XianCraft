use crate::world_model::{ItemZoneData, MultiZoneData, NpcPrototype, NpcZoneData, Room, WorldConfig, ZoneData, Quest, QuestRegistry, SkillRegistry, MonsterRegistry};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

// A container for all static world data loaded from JSON files.
#[derive(Debug)]
pub struct StaticWorldData {
    pub config: WorldConfig,
    pub rooms: HashMap<String, Room>, // Global mapping from room_id to Room
    pub npc_prototypes: HashMap<u32, NpcPrototype>,
    pub item_prototypes: HashMap<u32, crate::world_model::ItemPrototype>,
    pub quests: HashMap<String, Quest>,
    pub skills: HashMap<String, crate::world_model::SkillTemplate>,
    pub monsters: HashMap<String, crate::world_model::MonsterTemplate>,
}

pub fn load_all_data(base_path: &str) -> Result<StaticWorldData> {
    let path = Path::new(base_path);

    // 1. Load world_config.json
    let config_str = fs::read_to_string(path.join("world_config.json"))
        .context("Failed to read world_config.json")?;
    let config: WorldConfig = serde_json::from_str(&config_str)?;

    // 2. Load all rooms from the maps directory
    let mut rooms = HashMap::new();
    let maps_dir = path.join("maps");
    if maps_dir.is_dir() {
        for entry in fs::read_dir(maps_dir)? {
            let entry = entry?;
            let zone_str = fs::read_to_string(entry.path())?;
            
            // Try parsing as single ZoneData first, then MultiZoneData
            if let Ok(zone) = serde_json::from_str::<ZoneData>(&zone_str) {
                for room in zone.rooms {
                    rooms.insert(room.id.clone(), room);
                }
            } else if let Ok(multi_zone) = serde_json::from_str::<MultiZoneData>(&zone_str) {
                for zone in multi_zone.zones {
                    for room in zone.rooms {
                        rooms.insert(room.id.clone(), room);
                    }
                }
            } else {
                return Err(anyhow::anyhow!("Failed to parse zone data from {:?}", entry.path()));
            }
        }
    }

    // 3. Load all NPC prototypes and Item prototypes from the data directory
    let mut npc_prototypes = HashMap::new();
    let mut item_prototypes = HashMap::new();
    let mut quests = HashMap::new();
    let mut skills = HashMap::new();
    let mut monsters = HashMap::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let file_name = entry.file_name().into_string().unwrap_or_default();

        // A simple convention: load any file that ends with 'npcs.json' or 'monster_registry.json'
        if file_name.ends_with("npcs.json") || file_name == "monster_registry.json" {
            let npc_str = fs::read_to_string(entry.path())
                .with_context(|| format!("Failed to read NPC/Monster file at {:?}", entry.path()))?;

            let npc_zone_data: NpcZoneData = serde_json::from_str(&npc_str)
                .with_context(|| format!("Failed to parse NPC/Monster data from {:?}", entry.path()))?;

            for (id, proto) in npc_zone_data.entities {
                npc_prototypes.insert(id, proto);
            }
        } else if file_name.ends_with("items.json") {
            let item_str = fs::read_to_string(entry.path())
                .with_context(|| format!("Failed to read Item file at {:?}", entry.path()))?;

            let item_zone_data: ItemZoneData = serde_json::from_str(&item_str)
                .with_context(|| format!("Failed to parse Item zone data from {:?}", entry.path()))?;

            for (id, proto) in item_zone_data.items {
                item_prototypes.insert(id, proto);
            }
        } else if file_name == "quest_registry.json" {
            let quest_str = fs::read_to_string(entry.path())
                .with_context(|| format!("Failed to read Quest file at {:?}", entry.path()))?;

            let quest_registry: QuestRegistry = serde_json::from_str(&quest_str)
                .with_context(|| format!("Failed to parse Quest registry from {:?}", entry.path()))?;
            
            quests = quest_registry.quests;
        } else if file_name == "skills.json" {
            let skill_str = fs::read_to_string(entry.path())
                .with_context(|| format!("Failed to read Skills file at {:?}", entry.path()))?;

            let skill_registry: SkillRegistry = serde_json::from_str(&skill_str)
                .with_context(|| format!("Failed to parse Skill registry from {:?}", entry.path()))?;
            
            skills = skill_registry.skills;
        } else if file_name == "monsters.json" {
            let monster_str = fs::read_to_string(entry.path())
                .with_context(|| format!("Failed to read Monsters file at {:?}", entry.path()))?;

            let monster_registry: MonsterRegistry = serde_json::from_str(&monster_str)
                .with_context(|| format!("Failed to parse Monster registry from {:?}", entry.path()))?;
            
            monsters = monster_registry.monsters;
        }
    }

    println!(
        "✅ World data loaded: {} rooms, {} NPC prototypes, {} Item prototypes, {} quests, {} skills, {} monsters",
        rooms.len(),
        npc_prototypes.len(),
        item_prototypes.len(),
        quests.len(),
        skills.len(),
        monsters.len()
    );

    Ok(StaticWorldData {
        config,
        rooms,
        npc_prototypes,
        item_prototypes,
        quests,
        skills,
        monsters,
    })
}