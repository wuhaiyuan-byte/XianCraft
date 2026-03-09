#[cfg(test)]
mod tests;
pub mod command;
pub mod npc;
pub mod world;
pub mod world_model;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::SystemTime as StdSystemTime,
};
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use tower_http::services::ServeDir;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::command::{parse, Command};
use crate::world::player::{Player, PlayerQuestStatus};
use crate::world::world_state::WorldState;

#[derive(Deserialize)]
#[serde(tag = "type")]
enum ClientMessage {
    Login { user_id: String },
    Command { command: String },
}

#[derive(Serialize)]
#[serde(tag = "type")]
enum ServerMessage {
    Description { payload: String },
    Info { payload: String },
    Error { payload: String },
}

struct AppState {
    world_state: WorldState,
    player_sessions: Mutex<HashMap<u64, PlayerSession>>,
}

#[derive(Clone)]
struct PlayerSession {
    player: Player,
    user_id: Option<String>,
    sender: mpsc::Sender<Message>,
}

pub async fn run(world_state: WorldState) {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app_state = Arc::new(AppState {
        world_state,
        player_sessions: Mutex::new(HashMap::new()),
    });

    let game_loop_state = app_state.clone();
    tokio::spawn(async move {
        game_loop(game_loop_state).await;
    });

    // Use STATIC_DIR env var for the frontend assets, defaulting to "client/dist" for local dev.
    let static_dir = env::var("STATIC_DIR").unwrap_or_else(|_| "client/dist".to_string());
    info!("Serving static files from: {}", static_dir);

    let app = Router::new()
        .nest_service("/", ServeDir::new(static_dir))
        .route("/ws", get(websocket_handler))
        .with_state(app_state);

    let port = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap_or(3000);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("listening on {}", addr);
    axum::serve(tokio::net::TcpListener::bind(&addr).await.unwrap(), app.into_make_service())
        .await
        .unwrap();
}

// ... (The rest of the file remains the same)
