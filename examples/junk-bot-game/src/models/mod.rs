//! Data models layer
//!
//! Pure data structures without business logic

pub mod entities;
pub mod scenes;
pub mod game_context;
pub mod game_scene;
pub mod scene_helpers;

pub use game_context::GameContext;
pub use game_scene::{GameScene, GameState, handle_scene_input}; // GameState and handle_scene_input are auto-generated
pub use scene_helpers::proceed_to_next_floor;
