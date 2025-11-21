//! Save/Load plugin for ISSUN
//!
//! This plugin provides comprehensive save/load functionality for games built with ISSUN.
//! It supports multiple save formats, auto-save functionality, and customizable hooks
//! for game-specific save/load behavior.
//!
//! # Quick Start
//!
//! ```ignore
//! use issun::builder::GameBuilder;
//! use issun::plugin::save_load::SaveLoadPlugin;
//!
//! let game = GameBuilder::new()
//!     .with_plugin(SaveLoadPlugin::new())
//!     .build()
//!     .await?;
//! ```
//!
//! # Features
//!
//! - **Multiple Save Formats**: JSON and RON format support
//! - **Auto-Save**: Configurable automatic saving at intervals or checkpoints
//! - **Hook System**: Customize save/load behavior with custom hooks
//! - **File Management**: List, delete, and inspect save files
//! - **Error Handling**: Comprehensive error reporting and recovery
//! - **Metadata Support**: Rich save file metadata including timestamps and versions
//!
//! # Save Operations
//!
//! The plugin supports several types of save operations through events:
//! - Manual saves triggered by player action
//! - Auto-saves triggered by time intervals or game events
//! - Quick saves and checkpoint saves
//!
//! # Load Operations
//!
//! Load operations include:
//! - Loading specific save slots
//! - Continue from last save
//! - Save file validation and error recovery
//!
//! # Custom Formats
//!
//! You can extend the plugin to support additional save formats by implementing
//! the `SaveRepository` trait and registering it with the plugin.
//!
//! # Hook Customization
//!
//! Implement the `SaveLoadHook` trait to add custom behavior:
//! - Data validation and preprocessing
//! - Cloud backup integration
//! - Save file compression or encryption
//! - Progress tracking and achievements
//! - Custom error handling
//!
//! # Example Usage
//!
//! ```ignore
//! use issun::plugin::save_load::{
//!     SaveLoadPlugin, SaveLoadConfig, SaveFormat,
//!     SaveGameRequested, LoadGameRequested, AutoSaveRequested
//! };
//! use issun::event::EventBus;
//! use std::path::PathBuf;
//!
//! // Configure the plugin
//! let config = SaveLoadConfig {
//!     save_directory: PathBuf::from("my_game_saves"),
//!     format: SaveFormat::Ron,
//!     enable_auto_save: true,
//!     auto_save_interval: 300, // 5 minutes
//! };
//!
//! let game = GameBuilder::new()
//!     .with_plugin(
//!         SaveLoadPlugin::new()
//!             .with_config(config)
//!     )
//!     .build()
//!     .await?;
//!
//! // Later in your game logic...
//! let event_bus = /* get event bus from context */;
//!
//! // Save the game
//! let save_event = SaveGameRequested {
//!     slot: "player_save_1".to_string(),
//!     label: Some("Level 5 Complete".to_string()),
//! };
//! event_bus.publish(save_event).await?;
//!
//! // Load the game
//! let load_event = LoadGameRequested {
//!     slot: "player_save_1".to_string(),
//! };
//! event_bus.publish(load_event).await?;
//!
//! // Trigger auto-save
//! let auto_save_event = AutoSaveRequested {
//!     reason: Some("checkpoint".to_string()),
//! };
//! event_bus.publish(auto_save_event).await?;
//! ```

mod events;
mod hook;
mod plugin;
mod system;

// Re-export public API
pub use events::*;
pub use hook::{DefaultSaveLoadHook, SaveLoadHook};
pub use plugin::{SaveFormat, SaveLoadConfig, SaveLoadPlugin};
pub use system::SaveLoadSystem;
