//! Events for faction system

use crate::event::Event;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::types::{FactionId, Outcome, OperationId};

// ========================================
// Command Events (Request)
// ========================================

/// Request to launch an operation (Command Event)
///
/// This is a "command" event that requests an operation to be launched.
/// `FactionSystem` processes this and publishes `OperationLaunchedEvent`.
///
/// # Example
///
/// ```ignore
/// use issun::event::EventBus;
/// use issun::plugin::faction::{OperationLaunchRequested, FactionId};
/// use serde_json::json;
///
/// let mut bus = resources.get_mut::<EventBus>().await.unwrap();
/// bus.publish(OperationLaunchRequested {
///     faction_id: FactionId::new("crimson-syndicate"),
///     operation_name: "Capture Nova Harbor".into(),
///     metadata: json!({
///         "target": "nova-harbor",
///         "troops": 50,
///         "strategy": "stealth"
///     }),
/// });
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationLaunchRequested {
    /// Faction launching the operation
    pub faction_id: FactionId,
    /// Display name for the operation
    pub operation_name: String,
    /// Game-specific operation data
    #[serde(default)]
    pub metadata: serde_json::Value,
}

impl Event for OperationLaunchRequested {}

/// Request to resolve an operation (Command Event)
///
/// This is a "command" event that requests an operation to be resolved.
/// `FactionSystem` processes this and publishes either `OperationCompletedEvent` or `OperationFailedEvent`.
///
/// # Example
///
/// ```ignore
/// use issun::event::EventBus;
/// use issun::plugin::faction::{OperationResolveRequested, OperationId, Outcome};
/// use std::collections::HashMap;
/// use serde_json::json;
///
/// let mut bus = resources.get_mut::<EventBus>().await.unwrap();
/// bus.publish(OperationResolveRequested {
///     operation_id: OperationId::new("op-001"),
///     outcome: Outcome {
///         operation_id: OperationId::new("op-001"),
///         success: true,
///         metrics: HashMap::from([
///             ("casualties".into(), 5.0),
///             ("control_gained".into(), 0.15),
///         ]),
///         metadata: json!({ "notes": "Minimal resistance" }),
///     },
/// });
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationResolveRequested {
    /// Operation to resolve
    pub operation_id: OperationId,
    /// Result of the operation
    pub outcome: Outcome,
}

impl Event for OperationResolveRequested {}

// ========================================
// State Events (Notification)
// ========================================

/// Published when an operation is launched (State Change Event)
///
/// This event is published after an operation has been launched.
/// It represents a **confirmed state change** and can be replicated over network.
///
/// # Example
///
/// ```ignore
/// use issun::event::EventBus;
/// use issun::plugin::faction::OperationLaunchedEvent;
///
/// // In a system
/// let mut bus = resources.get_mut::<EventBus>().await.unwrap();
/// let reader = bus.reader::<OperationLaunchedEvent>();
///
/// for event in reader.iter() {
///     println!(
///         "Faction {} launched operation {}",
///         event.faction_id.as_str(),
///         event.operation_name
///     );
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationLaunchedEvent {
    /// Operation that was launched
    pub operation_id: OperationId,
    /// Faction that launched the operation
    pub faction_id: FactionId,
    /// Display name of the operation
    pub operation_name: String,
}

impl Event for OperationLaunchedEvent {}

/// Published when an operation is completed (State Change Event)
///
/// This event is published after an operation has completed successfully.
/// It represents a **confirmed state change** and can be replicated over network.
///
/// # Example
///
/// ```ignore
/// use issun::event::EventBus;
/// use issun::plugin::faction::OperationCompletedEvent;
///
/// // In a system
/// let mut bus = resources.get_mut::<EventBus>().await.unwrap();
/// let reader = bus.reader::<OperationCompletedEvent>();
///
/// for event in reader.iter() {
///     println!(
///         "Faction {} completed operation {}: {}",
///         event.faction_id.as_str(),
///         event.operation_id.as_str(),
///         if event.success { "SUCCESS" } else { "FAILED" }
///     );
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationCompletedEvent {
    /// Operation that was completed
    pub operation_id: OperationId,
    /// Faction that completed the operation
    pub faction_id: FactionId,
    /// Whether the operation succeeded
    pub success: bool,
    /// Generic metrics from the outcome
    #[serde(default)]
    pub metrics: HashMap<String, f32>,
}

impl Event for OperationCompletedEvent {}

/// Published when an operation fails (State Change Event)
///
/// This event is published after an operation has failed.
/// It represents a **confirmed state change** and can be replicated over network.
///
/// # Example
///
/// ```ignore
/// use issun::event::EventBus;
/// use issun::plugin::faction::OperationFailedEvent;
///
/// // In a system
/// let mut bus = resources.get_mut::<EventBus>().await.unwrap();
/// let reader = bus.reader::<OperationFailedEvent>();
///
/// for event in reader.iter() {
///     println!(
///         "Faction {} operation {} failed: {}",
///         event.faction_id.as_str(),
///         event.operation_id.as_str(),
///         event.reason
///     );
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationFailedEvent {
    /// Operation that failed
    pub operation_id: OperationId,
    /// Faction whose operation failed
    pub faction_id: FactionId,
    /// Reason for failure
    pub reason: String,
}

impl Event for OperationFailedEvent {}
