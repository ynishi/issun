//! Events for action points system

use crate::event::Event;
use serde::{Deserialize, Serialize};

/// Published when an action is consumed
///
/// This event is published after `ActionPoints::consume_with()` is called
/// and can be used by other systems to react to action consumption.
///
/// # Example
///
/// ```ignore
/// use issun::event::EventBus;
/// use issun::plugin::action::ActionConsumedEvent;
///
/// // In a system
/// let mut bus = resources.get_mut::<EventBus>().await.unwrap();
/// let reader = bus.reader::<ActionConsumedEvent>();
///
/// for event in reader.iter() {
///     println!("Action '{}' consumed ({} remaining)", event.context, event.remaining);
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionConsumedEvent {
    /// What action was performed
    pub context: String,
    /// Actions remaining after consumption
    pub remaining: u32,
    /// Whether all actions are now depleted
    pub depleted: bool,
}

/// Published when actions are reset
///
/// This event is published by `ActionResetSystem` when action points
/// are reset to maximum (typically on day change).
///
/// # Example
///
/// ```ignore
/// use issun::event::EventBus;
/// use issun::plugin::action::ActionsResetEvent;
///
/// // In a system
/// let mut bus = resources.get_mut::<EventBus>().await.unwrap();
/// let reader = bus.reader::<ActionsResetEvent>();
///
/// for event in reader.iter() {
///     println!("Actions reset to {}", event.new_count);
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionsResetEvent {
    /// New action count after reset
    pub new_count: u32,
}

// Implement Event trait for both events
impl Event for ActionConsumedEvent {}
impl Event for ActionsResetEvent {}
