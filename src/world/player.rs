use std::collections::HashMap;

/// Holds the core attributes of a player.
#[derive(Debug, Clone)]
pub struct PlayerAttributes {
    pub strength: u32,
    pub dexterity: u32,
    pub constitution: u32,
    pub intelligence: u32,
}

/// Represents a player in the game world.
#[derive(Debug, Clone)]
pub struct Player {
    pub id: u64,
    pub name: String,
    pub qi: u32,
    pub max_qi: u32,
    pub neili: u32,
    pub max_neili: u32,
    pub attributes: PlayerAttributes,
    pub skills: HashMap<String, u32>,
    pub busy: u32, // Cooldown timer for actions
    pub aliases: HashMap<String, String>,
}

impl Player {
    /// Creates a new player with default stats.
    pub fn new(id: u64, name: String) -> Self {
        let attributes = PlayerAttributes {
            strength: 10,
            dexterity: 10,
            constitution: 10,
            intelligence: 10,
        };

        let mut player = Self {
            id,
            name,
            qi: 0, // Calculated below
            max_qi: 0, // Calculated below
            neili: 0, // Calculated below
            max_neili: 0, // Calculated below
            attributes,
            skills: HashMap::new(),
            busy: 0,
            aliases: HashMap::new(),
        };

        player.update_derived_stats();
        // Set current qi and neili to max after calculation
        player.qi = player.max_qi;
        player.neili = player.max_neili;
        
        player
    }
    
    /// Updates derived stats like max_qi and max_neili based on attributes.
    pub fn update_derived_stats(&mut self) {
        self.max_qi = self.calculate_max_qi();
        self.max_neili = self.calculate_max_neili();
    }

    /// Calculates max qi based on constitution.
    fn calculate_max_qi(&self) -> u32 {
        self.attributes.constitution * 10
    }

    /// Calculates max neili based on intelligence.
    fn calculate_max_neili(&self) -> u32 {
        self.attributes.intelligence * 5
    }

    /// Reduces player's qi by a certain amount.
    pub fn take_damage(&mut self, amount: u32) {
        self.qi = self.qi.saturating_sub(amount);
    }

    /// Heals the player by a certain amount.
    pub fn heal(&mut self, amount: u32) {
        self.qi = (self.qi + amount).min(self.max_qi);
    }

    /// Consumes neili if available. Returns true if successful.
    pub fn consume_neili(&mut self, amount: u32) -> bool {
        if self.neili >= amount {
            self.neili -= amount;
            true
        } else {
            false
        }
    }

    /// Checks if the player is alive.
    pub fn is_alive(&self) -> bool {
        self.qi > 0
    }

    /// Learns a new skill or updates an existing one.
    pub fn learn_skill(&mut self, skill_name: String, skill_level: u32) {
        self.skills.insert(skill_name, skill_level);
    }
}
