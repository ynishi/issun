//! Scene-specific data
//!
//! Each scene has its own data that is discarded on transition

mod card_selection;
mod combat;
mod drop_collection;
mod floor4_choice;
mod result;
mod room_selection;
mod title;

pub use card_selection::CardSelectionSceneData;
pub use combat::CombatSceneData;
pub use drop_collection::DropCollectionSceneData;
pub use floor4_choice::Floor4ChoiceSceneData;
pub use result::ResultSceneData;
pub use room_selection::RoomSelectionSceneData;
pub use title::TitleSceneData;
