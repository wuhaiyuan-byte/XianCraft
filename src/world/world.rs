use crate::npc::Npc;
use crate::world::room::{BaseRoom, Room};
use crate::world::world_loader;
use std::collections::HashMap;
use std::fs;
use std::sync::Mutex;

pub struct World {
    pub rooms: HashMap<String, Box<dyn Room>>,
    pub npcs: Mutex<HashMap<u64, Npc>>,
}

impl World {
    pub fn new() -> Self {
        let mut rooms: HashMap<String, Box<dyn Room>> = HashMap::new();
        let mut npcs = HashMap::new();
        let mut npc_id_counter: u64 = 0;

        let map_paths = fs::read_dir("data/maps").unwrap(); // Corrected path
        for path in map_paths {
            let path = path.unwrap().path();
            if path.is_file() {
                let map_config = world_loader::load_map(path.to_str().unwrap());
                for room_config in map_config.rooms {
                    let mut npc_ids = Vec::new();
                    // Handle both `npcs` and `spawns` for backward compatibility
                    if !room_config.npcs.is_empty() {
                        for npc_id in &room_config.npcs {
                            npc_ids.push(npc_id + npc_id_counter);
                        }
                    }
                    if !room_config.spawns.is_empty() {
                        for spawn in room_config.spawns {
                            npc_ids.push(spawn.npc + npc_id_counter);
                        }
                    }

                    let base_room = BaseRoom {
                        id: room_config.id.clone(),
                        name: room_config.name,
                        description: room_config.description,
                        exits: room_config.exits,
                        players: Default::default(),
                        npcs: Mutex::new(npc_ids),
                    };
                    rooms.insert(room_config.id.clone(), Box::new(base_room));
                }

                for mut npc_config in map_config.npcs {
                    npc_config.id += npc_id_counter;
                    let npc = Npc::new(
                        npc_config.id,
                        &npc_config.name,
                        &npc_config.description,
                        &npc_config.starting_room,
                    );
                    npcs.insert(npc.id, npc);
                }
                npc_id_counter += 100;
            }
        }

        // Validation
        for (room_id, room) in &rooms {
            for exit in room.get_exits().values() {
                if !rooms.contains_key(exit) {
                    panic!(
                        "Invalid map config: room '{}' has an exit to non-existent room '{}'",
                        room_id, exit
                    );
                }
            }
            for npc_id in room.get_npc_ids().iter() {
                if !npcs.contains_key(npc_id) {
                    panic!(
                        "Invalid map config: room '{}' contains non-existent NPC '{}'",
                        room_id, npc_id
                    );
                }
            }
        }

        Self {
            rooms,
            npcs: Mutex::new(npcs),
        }
    }

    pub fn get_room(&self, room_id: &str) -> Option<&Box<dyn Room>> {
        self.rooms.get(room_id)
    }
}
