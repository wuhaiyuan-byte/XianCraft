use crate::PlayerState;
use std::fmt::Debug;
use tokio::sync::mpsc;
use crate::ServerMessage;

/// The `Character` trait defines the common behavior for all entities in the world
/// that can think, act, and be interacted with, such as players and NPCs.
pub trait Character: Debug + Send + Sync {
    fn get_id(&self) -> usize;
    fn get_current_room_id(&self) -> usize;
    fn set_current_room_id(&mut self, room_id: usize);
    fn get_state(&self) -> &PlayerState;
    fn get_mut_state(&mut self) -> &mut PlayerState;
    fn get_sender(&self) -> mpsc::Sender<ServerMessage>;

    // We can add more complex behaviors later, e.g.:
    // fn think(&mut self, world: &World) -> Option<Action>;
    // fn on_damage(&mut self, amount: u32) -> bool; // returns true if dead
}

/// `PlayerCharacter` is the implementation of the `Character` trait for a
/// human-controlled player.
#[derive(Debug)]
pub struct PlayerCharacter {
    pub id: usize, // This will eventually be tied to a user account
    pub current_room_id: usize,
    pub state: PlayerState,

    /// The sender channel is used to send messages from the game world (like room descriptions
    /// or combat results) back to the player's WebSocket connection.
    pub sender: mpsc::Sender<ServerMessage>,
}

impl Character for PlayerCharacter {
    fn get_id(&self) -> usize {
        self.id
    }

    fn get_current_room_id(&self) -> usize {
        self.current_room_id
    }
    
    fn set_current_room_id(&mut self, room_id: usize) {
        self.current_room_id = room_id;
    }

    fn get_state(&self) -> &PlayerState {
        &self.state
    }

    fn get_mut_state(&mut self) -> &mut PlayerState {
        &mut self.state
    }

    fn get_sender(&self) -> mpsc::Sender<ServerMessage> {
        self.sender.clone()
    }
}
