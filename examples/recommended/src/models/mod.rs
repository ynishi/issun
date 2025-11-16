//! Data models layer
//!
//! Pure data structures without business logic

pub mod entities;
pub mod scenes;
pub mod game_context;
pub mod game_scene;

pub use game_context::GameContext;
pub use game_scene::GameScene;
