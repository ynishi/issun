//! Scene-specific data
//!
//! Each scene has its own data that is discarded on transition.

mod ping;
mod pong;
mod title;

pub use ping::PingSceneData;
pub use pong::PongSceneData;
pub use title::TitleSceneData;
