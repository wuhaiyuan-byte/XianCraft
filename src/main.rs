use anyhow::Result;
use server::world::world_loader::load_all_data;
use server::world::world_state::WorldState;
use std::env;
use std::sync::Arc;
use colored::control;

#[tokio::main]
async fn main() -> Result<()> {
    // Force colored output, even when not in a TTY.
    // This is crucial for the MUD client to receive colors.
    control::set_override(true);

    // Use the DATA_DIR environment variable if it's set, otherwise default to "./data".
    // This allows for flexible configuration between local dev and containerized deployment.
    let data_path = env::var("DATA_DIR").unwrap_or_else(|_| "./data".to_string());

    // Add a log to show which data path is being used, for easier debugging.
    println!("Loading game data from: {}", data_path);

    // Load all static data from the determined path.
    let static_world_data = Arc::new(load_all_data(&data_path)?);

    // Initialize the world state with the loaded static data.
    let world_state = WorldState::new(static_world_data);

    // Start the server with the initialized world state.
    server::run(world_state).await;

    Ok(())
}
