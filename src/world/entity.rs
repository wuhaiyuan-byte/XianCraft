use crate::world::player::Player;

/// Represents any entity that can exist in the game world.
#[derive(Debug)]
pub enum Entity {
    Player(Player),
    // Other entity types like NPC, Item, etc. can be added here
}
