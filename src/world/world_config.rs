use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Realm {
    pub name: String,
    pub level_required: u32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct WorldConfig {
    pub realms: Vec<Realm>,
}
