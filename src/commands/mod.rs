pub mod battle;
pub mod help;
pub mod interaction;
pub mod look;
pub mod movement;
pub mod player;
pub mod quest;
pub mod who;

pub use battle::{handle_attack, handle_cast, handle_kill};
pub use help::handle_help;
pub use interaction::{handle_get, handle_talk};
pub use look::handle_look;
pub use movement::handle_go;
pub use player::{handle_inventory, handle_rest, handle_score, handle_work};
pub use quest::{handle_accept, handle_quest};
pub use who::handle_who;
