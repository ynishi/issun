//! Data models layer
//!
//! Pure data structures without business logic

pub mod entities;
pub mod game_context;
pub mod game_scene;
pub mod scene_helpers;
pub mod scenes;

pub use game_context::GameContext;
pub use game_scene::{handle_scene_input, GameScene}; // handle_scene_input is auto-generated
pub use scene_helpers::proceed_to_next_floor;
