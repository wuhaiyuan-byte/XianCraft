use crate::npc::Npc;
use crate::world::world_loader::StaticWorldData;
use crate::world_model::Room;
use rand::seq::SliceRandom;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Represents the dynamic, mutable state of the game world.
pub struct DynamicData {
    pub players: HashMap<u64, PlayerLocation>,
    pub npcs: HashMap<u64, Npc>, // Instance of an NPC in the world
    pub room_items: HashMap<String, Vec<u32>>,
    pub next_npc_id: u64,
}

// Tracks a player's location.
#[derive(Clone)]
pub struct PlayerLocation {
    pub room_id: String,
    pub user_name: Option<String>,
}

// The main world state, combining static and dynamic data.
#[derive(Clone)]
pub struct WorldState {
    pub static_data: Arc<StaticWorldData>,
    pub dynamic_data: Arc<Mutex<DynamicData>>,
}

impl WorldState {
    pub fn new(static_data: Arc<StaticWorldData>) -> Self {
        let mut room_items = HashMap::new();
        for (room_id, room) in &static_data.rooms {
            room_items.insert(room_id.clone(), room.items.clone());
        }

        let mut dynamic_data = DynamicData {
            players: HashMap::new(),
            npcs: HashMap::new(),
            room_items,
            next_npc_id: 0,
        };

        // Spawn initial NPCs from prototypes
        for room in static_data.rooms.values() {
            for npc_prototype_id in &room.npcs {
                if let Some(prototype) = static_data.npc_prototypes.get(npc_prototype_id) {
                    let mut npc = Npc::from_prototype(
                        dynamic_data.next_npc_id,
                        *npc_prototype_id,
                        prototype,
                        room.id.clone(),
                    );

                    // Try to load combat stats from monsters.json using monster name
                    let monster_key = npc.name.clone();
                    let mut found = false;
                    for monster in static_data.monsters.values() {
                        if monster.name == monster_key {
                            npc.init_combat_stats(
                                monster.max_hp as i32,
                                monster.attack as i32,
                                monster.defense as i32,
                            );
                            found = true;
                            break;
                        }
                    }
                    if !found {
                        // Default combat stats if no monster template found
                        npc.init_combat_stats(50, 10, 2);
                    }

                    dynamic_data.npcs.insert(dynamic_data.next_npc_id, npc);
                    dynamic_data.next_npc_id += 1;
                }
            }
        }

        println!("✅ Spawned {} NPC instances.", dynamic_data.npcs.len());

        Self {
            static_data,
            dynamic_data: Arc::new(Mutex::new(dynamic_data)),
        }
    }

    pub fn get_room(&self, room_id: &str) -> Option<&Room> {
        self.static_data.rooms.get(room_id)
    }

    pub fn move_player_to_room(&self, player_id: u64, to_room_id: &str, user_name: Option<String>) {
        let mut data = self.dynamic_data.lock().unwrap();
        data.players.insert(
            player_id,
            PlayerLocation {
                room_id: to_room_id.to_string(),
                user_name,
            },
        );
    }

    pub fn get_players_in_room(&self, room_id: &str, exclude_player_id: u64) -> Vec<String> {
        let data = self.dynamic_data.lock().unwrap();
        data.players
            .iter()
            .filter(|(id, loc)| **id != exclude_player_id && loc.room_id == room_id)
            .filter_map(|(_, loc)| loc.user_name.clone())
            .collect()
    }

    pub fn update_player_name(&self, player_id: u64, user_name: String) {
        let mut data = self.dynamic_data.lock().unwrap();
        if let Some(loc) = data.players.get_mut(&player_id) {
            loc.user_name = Some(user_name);
        }
    }

    pub fn get_player_room_id(&self, player_id: u64) -> Option<String> {
        self.dynamic_data
            .lock()
            .unwrap()
            .players
            .get(&player_id)
            .map(|loc| loc.room_id.clone())
    }

    pub fn get_npcs_in_room(&self, room_id: &str) -> Vec<Npc> {
        let data = self.dynamic_data.lock().unwrap();
        data.npcs
            .values()
            .filter(|npc| npc.current_room == room_id)
            .cloned()
            .collect()
    }

    pub fn get_items_in_room(&self, room_id: &str) -> Vec<u32> {
        let data = self.dynamic_data.lock().unwrap();
        data.room_items.get(room_id).cloned().unwrap_or_default()
    }

    pub fn add_item_to_room(&self, room_id: &str, item_id: u32) {
        let mut data = self.dynamic_data.lock().unwrap();
        data.room_items
            .entry(room_id.to_string())
            .or_default()
            .push(item_id);
    }

    pub fn remove_item_from_room(&self, room_id: &str, item_id: u32) -> bool {
        let mut data = self.dynamic_data.lock().unwrap();
        if let Some(items) = data.room_items.get_mut(room_id) {
            if let Some(pos) = items.iter().position(|&x| x == item_id) {
                items.remove(pos);
                return true;
            }
        }
        false
    }

    pub fn respawn_monsters(&self) {
        let mut data = self.dynamic_data.lock().unwrap();
        let mut new_npcs = Vec::new();

        for room in self.static_data.rooms.values() {
            // Check if room has npcs and if it is a wild/respawn zone
            if room.npcs.is_empty() {
                continue;
            }

            // Count current npcs in this room
            let current_count = data
                .npcs
                .values()
                .filter(|n| n.current_room == room.id)
                .count();

            // If empty, respawn one random monster from the room's list
            if current_count == 0 {
                if let Some(&proto_id) = room.npcs.choose(&mut rand::thread_rng()) {
                    if let Some(prototype) = self.static_data.npc_prototypes.get(&proto_id) {
                        let npc = Npc::from_prototype(
                            data.next_npc_id,
                            proto_id,
                            prototype,
                            room.id.clone(),
                        );
                        new_npcs.push((data.next_npc_id, npc));
                        data.next_npc_id += 1;
                    }
                }
            }
        }

        for (id, npc) in new_npcs {
            data.npcs.insert(id, npc);
        }
    }

    pub fn tick(&self) {
        // Handle periodic world updates
        self.respawn_monsters();
    }
}
