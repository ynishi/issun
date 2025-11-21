//! Faction management plugin for strategy, RPG, and simulation games

mod events;
mod factions;
mod hook;
mod plugin;
mod service;
mod state;
mod system;
mod types;

pub use events::{
    OperationCompletedEvent, OperationFailedEvent, OperationLaunchRequested,
    OperationLaunchedEvent, OperationResolveRequested,
};
pub use factions::Factions;
pub use hook::{DefaultFactionHook, FactionHook};
pub use plugin::FactionPlugin;
pub use service::FactionService;
pub use state::FactionState;
pub use system::FactionSystem;
pub use types::{
    Faction, FactionError, FactionId, Operation, OperationId, OperationStatus, Outcome,
};
