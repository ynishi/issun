//! Faction management plugin for strategy, RPG, and simulation games

mod events;
mod hook;
mod plugin;
mod registry;
mod service;
mod system;
mod types;

pub use events::{
    OperationCompletedEvent, OperationFailedEvent, OperationLaunchRequested,
    OperationLaunchedEvent, OperationResolveRequested,
};
pub use hook::{DefaultFactionHook, FactionHook};
pub use plugin::FactionPlugin;
pub use registry::FactionRegistry;
pub use service::FactionService;
pub use system::FactionSystem;
pub use types::{
    Faction, FactionError, FactionId, Operation, OperationId, OperationStatus, Outcome,
};
