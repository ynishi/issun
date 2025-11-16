//! Data models layer
//!
//! Pure data structures without business logic

pub mod entities;
pub mod scenes;
pub mod game_context;
pub mod game_scene;
pub mod game_state;
pub mod scene_handler;

pub use game_context::GameContext;
pub use game_scene::GameScene;
pub use game_state::GameState;
pub use scene_handler::handle_scene_input;
