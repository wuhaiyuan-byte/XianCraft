use anyhow::Result;
use server::world::loader::load_all_data;
use server::world::world_state::WorldState;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Load all static data from the "data" directory
    let static_world_data = Arc::new(load_all_data("./data")?);

    // Initialize the world state with the loaded static data
    let world_state = WorldState::new(static_world_data);

    // Start the server with the initialized world state
    server::run(world_state).await;

    Ok(())
}
