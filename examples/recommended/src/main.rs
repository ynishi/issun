//! ISSUN Recommended Project Structure Example
//!
//! This is a template project showing the recommended structure for ISSUN games.
//! Copy this structure to start your own game!

mod models;
mod systems;
mod assets;
mod game;
mod ui;

use issun::prelude::*;

#[tokio::main]
async fn main() {
    println!("ISSUN Recommended Structure Example");
    println!("====================================");
    println!();
    println!("This is a template project. Key files:");
    println!("  - src/models/     : Data models (entities, scenes, context)");
    println!("  - src/systems/    : Business logic");
    println!("  - src/assets/     : Game content data");
    println!("  - src/game/       : Game-specific coordinators");
    println!("  - src/ui/         : UI rendering");
    println!();
    println!("Copy this structure to create your own game!");
}
