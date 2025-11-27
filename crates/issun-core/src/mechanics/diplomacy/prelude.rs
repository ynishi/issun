//! Prelude for diplomacy mechanics.

pub use super::mechanic::DiplomacyMechanic;
pub use super::policies::{ContextPolicy, InfluencePolicy, ResistancePolicy};
pub use super::strategies::{LinearInfluence, NoContext, SkepticalResistance};
pub use super::types::{ArgumentType, DiplomacyConfig, DiplomacyEvent, DiplomacyInput, DiplomacyState};
