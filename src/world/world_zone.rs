use super::world_room::Room;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug, Clone)]
pub struct Zone {
    pub id: String,
    pub name: String,
    pub rooms: HashMap<u32, Room>,
}
