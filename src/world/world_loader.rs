use crate::npc::NpcPrototype;
use crate::world::{
    item::ItemPrototype,
    world_config::WorldConfig,
    zone::Zone,
};
use std::{collections::HashMap, fs};

pub struct WorldData {
    pub world_config: WorldConfig,
    pub item_prototypes: HashMap<u32, ItemPrototype>,
    pub npc_prototypes: HashMap<u32, NpcPrototype>,
    pub zones: HashMap<String, Zone>,
}

pub fn load_world_data() -> WorldData {
    let world_config = load_world_config("data/world_config.json");
    let item_prototypes = load_item_prototypes("data/entities/items.json");
    let npc_prototypes = load_npc_prototypes("data/entities/npcs.json");
    let zones = load_zones("data/maps");

    WorldData {
        world_config,
        item_prototypes,
        npc_prototypes,
        zones,
    }
}

fn load_world_config(path: &str) -> WorldConfig {
    let data = fs::read_to_string(path).expect("Unable to read world config file");
    serde_json::from_str(&data).unwrap_or_else(|e| {
        panic!("Failed to parse world config file at {}: {}", path, e);
    })
}

fn load_item_prototypes(path: &str) -> HashMap<u32, ItemPrototype> {
    let data = fs::read_to_string(path).expect("Unable to read item prototypes file");
    let prototypes: HashMap<String, ItemPrototype> = serde_json::from_str(&data).unwrap_or_else(|e| {
        panic!("Failed to parse item prototypes file at {}: {}", path, e);
    });

    prototypes
        .into_iter()
        .map(|(k, v)| (k.parse::<u32>().unwrap(), v))
        .collect()
}

fn load_npc_prototypes(path: &str) -> HashMap<u32, NpcPrototype> {
    let data = fs::read_to_string(path).expect("Unable to read npc prototypes file");
    let prototypes: HashMap<String, NpcPrototype> = serde_json::from_str(&data).unwrap_or_else(|e| {
        panic!("Failed to parse npc prototypes file at {}: {}", path, e);
    });

    prototypes
        .into_iter()
        .map(|(k, v)| (k.parse::<u32>().unwrap(), v))
        .collect()
}

fn load_zones(path: &str) -> HashMap<String, Zone> {
    let mut zones = HashMap::new();
    let paths = fs::read_dir(path).unwrap();

    for path in paths {
        let path = path.unwrap().path();
        if path.is_file() {
            let data = fs::read_to_string(&path).expect("Unable to read zone file");
            let zone: Zone = serde_json::from_str(&data).unwrap_or_else(|e| {
                panic!(
                    "Failed to parse zone file at {}: {}",
                    path.to_str().unwrap(),
                    e
                );
            });
            zones.insert(zone.id.clone(), zone);
        }
    }
    zones
}
