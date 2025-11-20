//! Events for territory system

use crate::event::Event;
use serde::{Deserialize, Serialize};

use super::types::{TerritoryEffects, TerritoryId};

/// Request to change territory control (Command Event)
///
/// This is a "command" event that requests a state change.
/// `TerritorySystem` processes this and publishes `TerritoryControlChangedEvent`.
///
/// # Example
///
/// ```ignore
/// use issun::event::EventBus;
/// use issun::plugin::territory::{TerritoryControlChangeRequested, TerritoryId};
///
/// let mut bus = resources.get_mut::<EventBus>().await.unwrap();
/// bus.publish(TerritoryControlChangeRequested {
///     id: TerritoryId::new("nova-harbor"),
///     delta: 0.1,  // Increase control by 10%
/// });
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerritoryControlChangeRequested {
    /// Territory to modify
    pub id: TerritoryId,
    /// Change in control (can be negative)
    pub delta: f32,
}

impl Event for TerritoryControlChangeRequested {}

/// Request to develop a territory (Command Event)
///
/// This is a "command" event that requests development.
/// `TerritorySystem` processes this and publishes `TerritoryDevelopedEvent`.
///
/// # Example
///
/// ```ignore
/// use issun::event::EventBus;
/// use issun::plugin::territory::{TerritoryDevelopmentRequested, TerritoryId};
///
/// let mut bus = resources.get_mut::<EventBus>().await.unwrap();
/// bus.publish(TerritoryDevelopmentRequested {
///     id: TerritoryId::new("nova-harbor"),
/// });
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerritoryDevelopmentRequested {
    /// Territory to develop
    pub id: TerritoryId,
}

impl Event for TerritoryDevelopmentRequested {}

/// Published when territory control changes (State Change Event)
///
/// This event is published after territory control has changed.
/// It represents a **confirmed state change** and can be replicated over network.
///
/// # Example
///
/// ```ignore
/// use issun::event::EventBus;
/// use issun::plugin::territory::TerritoryControlChangedEvent;
///
/// // In a system
/// let mut bus = resources.get_mut::<EventBus>().await.unwrap();
/// let reader = bus.reader::<TerritoryControlChangedEvent>();
///
/// for event in reader.iter() {
///     println!(
///         "Territory {} control: {:.1}% → {:.1}%",
///         event.id.as_str(),
///         event.old_control * 100.0,
///         event.new_control * 100.0
///     );
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerritoryControlChangedEvent {
    /// Territory that changed
    pub id: TerritoryId,
    /// Control before change
    pub old_control: f32,
    /// Control after change
    pub new_control: f32,
    /// Actual delta applied (may differ from requested due to clamping)
    pub delta: f32,
}

impl Event for TerritoryControlChangedEvent {}

/// Published when territory is developed (State Change Event)
///
/// This event is published after territory development level increases.
/// It represents a **confirmed state change** and can be replicated over network.
///
/// # Example
///
/// ```ignore
/// use issun::event::EventBus;
/// use issun::plugin::territory::TerritoryDevelopedEvent;
///
/// // In a system
/// let mut bus = resources.get_mut::<EventBus>().await.unwrap();
/// let reader = bus.reader::<TerritoryDevelopedEvent>();
///
/// for event in reader.iter() {
///     println!(
///         "Territory {} developed: Lv{} → Lv{}",
///         event.id.as_str(),
///         event.old_level,
///         event.new_level
///     );
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerritoryDevelopedEvent {
    /// Territory that was developed
    pub id: TerritoryId,
    /// Development level before
    pub old_level: u32,
    /// Development level after
    pub new_level: u32,
}

impl Event for TerritoryDevelopedEvent {}

/// Published when territory effects are updated (State Change Event)
///
/// This event is published when territory effects are recalculated
/// (e.g., due to policy changes, neighbor bonuses, etc.).
///
/// # Example
///
/// ```ignore
/// use issun::event::EventBus;
/// use issun::plugin::territory::TerritoryEffectsUpdatedEvent;
///
/// // In a system
/// let mut bus = resources.get_mut::<EventBus>().await.unwrap();
/// let reader = bus.reader::<TerritoryEffectsUpdatedEvent>();
///
/// for event in reader.iter() {
///     println!(
///         "Territory {} effects updated: income {:.1}x, cost {:.1}x",
///         event.id.as_str(),
///         event.effects.income_multiplier,
///         event.effects.cost_multiplier
///     );
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerritoryEffectsUpdatedEvent {
    /// Territory that was updated
    pub id: TerritoryId,
    /// New effects
    pub effects: TerritoryEffects,
}

impl Event for TerritoryEffectsUpdatedEvent {}
