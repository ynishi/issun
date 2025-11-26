//! Save/Load plugin for persistent game state
//!
//! This plugin provides:
//! - Declarative persistence via `#[require(Save)]` component marker
//! - Type-safe serialization using Bevy's Reflect system
//! - Automatic entity reference mapping (Entity IDs preserved across save/load)
//! - Model/View separation (save game logic, not visuals)
//! - Multiple save slots with metadata
//! - Auto-save functionality
//! - Error handling and notifications
//!
//! # Architecture
//!
//! The save/load plugin wraps [moonshine_save](https://github.com/Zeenobit/moonshine_save):
//! - **moonshine_save**: Core save/load functionality (serialization, entity mapping)
//! - **ISSUN Wrapper**: Ergonomic commands, slot management, validation, error handling
//!
//! # Usage Example
//!
//! ```ignore
//! use bevy::prelude::*;
//! use issun_bevy::plugins::save_load::{
//!     SaveLoadPlugin, SaveLoadConfig,
//!     Save, Unload,
//!     SaveRequested, LoadRequested,
//!     SaveMetadata,
//! };
//!
//! // Register the plugin
//! App::new()
//!     .add_plugins(SaveLoadPlugin::default())
//!     .add_systems(Startup, setup)
//!     .add_systems(Update, save_game)
//!     .run();
//!
//! // Mark components for saving
//! #[derive(Component, Reflect)]
//! #[reflect(Component)]
//! #[require(Save)]  // ← Makes this component saveable!
//! struct Player {
//!     name: String,
//!     level: u32,
//! }
//!
//! // Mark visual components for despawn before load
//! #[derive(Component, Reflect)]
//! #[reflect(Component)]
//! #[require(Unload)]  // ← Will be despawned before load
//! struct Sprite {
//!     texture: Handle<Image>,
//! }
//!
//! // Save game
//! fn save_game(mut commands: Commands, keyboard: Res<ButtonInput<KeyCode>>) {
//!     if keyboard.just_pressed(KeyCode::F5) {
//!         commands.write_message(SaveRequested {
//!             slot_name: "quicksave".into(),
//!             metadata: None, // Auto-generated
//!         });
//!     }
//! }
//!
//! // Load game
//! fn load_game(mut commands: Commands, keyboard: Res<ButtonInput<KeyCode>>) {
//!     if keyboard.just_pressed(KeyCode::F9) {
//!         commands.write_message(LoadRequested {
//!             slot_name: "quicksave".into(),
//!         });
//!     }
//! }
//!
//! // Listen to save/load events
//! fn on_save_completed(mut messages: MessageReader<SaveCompleted>) {
//!     for msg in messages.read() {
//!         info!("Game saved to slot: {}", msg.slot_name);
//!     }
//! }
//!
//! fn on_load_completed(mut messages: MessageReader<LoadCompleted>) {
//!     for msg in messages.read() {
//!         info!("Game loaded from slot: {}", msg.slot_name);
//!     }
//! }
//! ```
//!
//! # Model/View Separation
//!
//! **Save only game logic, not visuals:**
//! - Components with `#[require(Save)]` are persisted
//! - Components with `#[require(Unload)]` are despawned before load
//! - Visual entities should be respawned after load based on saved data
//!
//! # File Format
//!
//! Saves are stored as RON (Rusty Object Notation) files in the save directory:
//! ```text
//! ./saves/
//! ├── slot_1.ron       (manual save)
//! ├── slot_2.ron       (manual save)
//! ├── auto_save.ron    (auto-save)
//! └── quicksave.ron    (quicksave)
//! ```

mod components;
mod events;
mod plugin;
mod resources;
mod systems;

// Re-export public API
pub use components::{Save, SaveMetadata, Unload};
pub use events::*;
pub use plugin::SaveLoadPlugin;
pub use resources::{SaveLoadConfig, SaveSlotInfo, SaveSlotRegistry};
pub use systems::*;
