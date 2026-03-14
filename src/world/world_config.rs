use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Realm {
    pub name: String,
    pub level_required: u32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct WorldConfig {
    pub realms: Vec<Realm>,
    #[serde(default = "default_combat_tick_ms")]
    pub combat_tick_ms: u64,
    #[serde(default = "default_player_display_prefix")]
    pub player_display_prefix: String,
}

fn default_combat_tick_ms() -> u64 {
    500
}

fn default_player_display_prefix() -> String {
    "修士".to_string()
}
