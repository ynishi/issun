//! Scene-specific data
//!
//! Each scene has its own data that is discarded on transition

mod title;
mod combat;
mod result;

pub use title::TitleSceneData;
pub use combat::CombatSceneData;
pub use result::ResultSceneData;
