//! Scene-specific data
//!
//! Each scene has its own data that is discarded on transition

mod title;
mod combat;
mod result;
mod drop_collection;
mod card_selection;
mod room_selection;
mod floor4_choice;

pub use title::TitleSceneData;
pub use combat::{CombatSceneData, EquipTarget};
pub use result::ResultSceneData;
pub use drop_collection::DropCollectionSceneData;
pub use card_selection::CardSelectionSceneData;
pub use room_selection::RoomSelectionSceneData;
pub use floor4_choice::Floor4ChoiceSceneData;
