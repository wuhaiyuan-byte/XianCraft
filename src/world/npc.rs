use std::collections::HashMap;

/// Represents a Non-Player Character in the game.
#[derive(Debug)]
pub struct Npc {
    pub id: u64,
    pub name: String,
    pub qi: u32,
    pub max_qi: u32,
    pub skills: HashMap<String, u32>,
}
