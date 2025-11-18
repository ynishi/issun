//! Data models layer
//!
//! Pure data structures without business logic

pub mod entities;
pub mod game_context;
pub mod game_scene;
pub mod ping_pong;
pub mod scenes;

pub use game_context::GameContext;
pub use game_scene::{handle_scene_input, GameScene}; // handle_scene_input is auto-generated
pub use ping_pong::{PingPongMessageDeck, PingPongStage};
