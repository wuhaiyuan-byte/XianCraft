use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Mutex;

/// The `Room` trait defines the common behavior for all types of rooms in the world.
/// By using a trait, we can have different kinds of rooms (e.g., shops, traps, standard rooms)
/// that can all be treated the same way by the game engine.
pub trait Room: Debug + Send + Sync {
    /// Returns the unique identifier for the room.
    fn get_id(&self) -> &str;

    /// Returns the name of the room.
    fn get_name(&self) -> &str;

    /// Returns the main description of the room, which a player sees upon entering or looking.
    fn get_description(&self) -> &str;

    /// Returns a map of available exits from this room.
    /// The key is the direction (e.g., "north"), and the value is the ID of the connecting room.
    fn get_exits(&self) -> &HashMap<String, String>;

    fn get_player_ids(&self) -> Vec<u64>;

    fn get_npc_ids(&self) -> Vec<u64>;

    fn add_player(&self, player_id: u64);

    fn remove_player(&self, player_id: u64);
}

/// `BaseRoom` is a concrete implementation of a standard, simple room.
/// It holds the basic information: an ID, name, description, and a set of exits.
#[derive(Debug)]
pub struct BaseRoom {
    pub id: String,
    pub name: String,
    pub description: String,
    pub exits: HashMap<String, String>,
    pub players: Mutex<Vec<u64>>,
    pub npcs: Mutex<Vec<u64>>,
}

impl Room for BaseRoom {
    fn get_id(&self) -> &str {
        &self.id
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_description(&self) -> &str {
        &self.description
    }

    fn get_exits(&self) -> &HashMap<String, String> {
        &self.exits
    }

    fn get_player_ids(&self) -> Vec<u64> {
        self.players.lock().unwrap().clone()
    }

    fn get_npc_ids(&self) -> Vec<u64> {
        self.npcs.lock().unwrap().clone()
    }

    fn add_player(&self, player_id: u64) {
        self.players.lock().unwrap().push(player_id);
    }

    fn remove_player(&self, player_id: u64) {
        self.players.lock().unwrap().retain(|&id| id != player_id);
    }
}
