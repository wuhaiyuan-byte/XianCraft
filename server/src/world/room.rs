use std::collections::HashMap;
use std::fmt::Debug;

/// The `Room` trait defines the common behavior for all types of rooms in the world.
/// By using a trait, we can have different kinds of rooms (e.g., shops, traps, standard rooms)
/// that can all be treated the same way by the game engine.
pub trait Room: Debug + Send + Sync {
    /// Returns the unique identifier for the room.
    fn get_id(&self) -> usize;

    /// Returns the name of the room.
    fn get_name(&self) -> &str;

    /// Returns the main description of the room, which a player sees upon entering or looking.
    fn get_description(&self) -> &str;

    /// Returns a map of available exits from this room.
    /// The key is the direction (e.g., "north"), and the value is the ID of the connecting room.
    fn get_exits(&self) -> &HashMap<String, usize>;

    // In the future, we could add more behaviors here, such as:
    // fn on_enter(&self, player: &mut Player, world: &World);
    // fn on_look(&self, player: &Player) -> String;
}

/// `BaseRoom` is a concrete implementation of a standard, simple room.
/// It holds the basic information: an ID, name, description, and a set of exits.
#[derive(Debug, Clone)]
pub struct BaseRoom {
    pub id: usize,
    pub name: String,
    pub description: String,
    pub exits: HashMap<String, usize>,
}

impl Room for BaseRoom {
    fn get_id(&self) -> usize {
        self.id
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_description(&self) -> &str {
        &self.description
    }

    fn get_exits(&self) -> &HashMap<String, usize> {
        &self.exits
    }
}
