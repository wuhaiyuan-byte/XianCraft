# MUD Engine

This is a Rust-based MUD (Multi-User Dungeon) game server.

## Project Structure

This document outlines the file organization conventions for this project. Following these rules helps keep the codebase clean and maintainable.

### `/src` - Rust Source Code

This directory contains all the Rust source code for the game server.

-   **`main.rs`**: The main entry point of the application.
-   **`lib.rs`**: The core server logic, including the game loop and network handling.
-   **`src/world/`**: This module holds all game-world-related logic, such as rooms, NPCs, combat, and game state.
-   **Other modules**: Other `.rs` files in `src` should represent distinct logical components, like `command.rs` for parsing player commands.

### `/data` - Game Data and Configuration

This directory contains all non-code assets, primarily JSON files that define the game's content. This allows for easy modification of the game world without changing the source code.

-   **`data/maps/`**: Contains all map and zone definitions in `.json` format. Each file represents a specific area in the game world (e.g., `town.json`, `wudang.json`).
-   **`data/skills/`**: Contains skill definitions in `.json` format.

### `/client` - Frontend Code

This directory contains all the frontend code for the web client. This includes HTML, CSS, and JavaScript files.
