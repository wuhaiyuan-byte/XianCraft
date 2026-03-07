use crate::world::map_config::MapConfig;
use std::fs;

pub fn load_map(path: &str) -> MapConfig {
    let data = fs::read_to_string(path).expect("Unable to read map file");
    let map_config: MapConfig = serde_json::from_str(&data).unwrap_or_else(|e| {
        panic!("Failed to parse map file at {}: {}", path, e);
    });
    map_config
}
