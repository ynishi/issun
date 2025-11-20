//! Shared data definitions for the Border Economy prototype.

pub mod context;
pub mod game_scene;
pub mod ids;
pub mod scenes;
pub mod vault;

pub use context::{GameContext, WeaponPrototypeState};
pub use game_scene::{handle_scene_input, GameScene};
pub use ids::{BudgetChannel, Currency, DemandProfile, FactionId, TerritoryId, WeaponPrototypeId};
pub use vault::{SlotEffect, SlotType, VaultInvestmentError, VaultOutcome, VaultStatus};
