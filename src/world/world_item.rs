use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct ItemPrototype {
    pub id: u32,
    pub name: String,
    pub description: String,
}
