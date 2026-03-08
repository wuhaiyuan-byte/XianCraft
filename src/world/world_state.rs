use crate::npc::Npc;
use crate::world::loader::StaticWorldData;
use crate::world_model::Room;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Represents the dynamic, mutable state of the game world.
pub struct DynamicData {
    pub players: HashMap<u64, PlayerLocation>,
    pub npcs: HashMap<u64, Npc>, // Instance of an NPC in the world
    pub room_items: HashMap<String, Vec<u32>>,
}

// Tracks a player's location.
#[derive(Clone)]
pub struct PlayerLocation {
    pub room_id: String,
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
        };

        // Spawn initial NPCs from prototypes
        let mut npc_instance_id: u64 = 0;
        for room in static_data.rooms.values() {
            for npc_prototype_id in &room.npcs {
                if let Some(prototype) = static_data.npc_prototypes.get(npc_prototype_id) {
                    let npc = Npc::from_prototype(
                        npc_instance_id, 
                        *npc_prototype_id, 
                        prototype, 
                        room.id.clone(),
                    );
                    dynamic_data.npcs.insert(npc_instance_id, npc);
                    npc_instance_id += 1;
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

    pub fn move_player_to_room(&self, player_id: u64, to_room_id: &str) {
        let mut data = self.dynamic_data.lock().unwrap();
        data.players.insert(player_id, PlayerLocation { room_id: to_room_id.to_string() });
    }

    pub fn get_player_room_id(&self, player_id: u64) -> Option<String> {
        self.dynamic_data.lock().unwrap().players.get(&player_id).map(|loc| loc.room_id.clone())
    }

    pub fn get_npcs_in_room(&self, room_id: &str) -> Vec<Npc> {
        let data = self.dynamic_data.lock().unwrap();
        data.npcs.values()
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
        data.room_items.entry(room_id.to_string()).or_default().push(item_id);
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
}