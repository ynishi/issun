//! Network-transparent EventBus for multiplayer games
//!
//! This module provides network support for the EventBus, allowing events to be
//! seamlessly transmitted across network nodes using the same API as local events.

#[cfg(feature = "network")]
pub mod types;

#[cfg(feature = "network")]
pub mod backend;

#[cfg(feature = "network")]
pub use types::{NetworkMetadata, NetworkScope, NetworkedEvent, NodeId};

#[cfg(feature = "network")]
pub use backend::NetworkBackend;
