//! Message types for contagion propagation events

use bevy::prelude::*;

// ==================== Commands (User Intent) ====================

/// Spawn a new contagion at a node
#[derive(Message, Clone, Reflect)]
#[reflect(opaque)]
pub struct ContagionSpawnRequested {
    pub contagion_id: String,
    pub content: super::components::ContagionContent,
    pub origin_node: Entity,
    pub mutation_rate: f32,
}

/// Trigger propagation step
#[derive(Message, Clone, Reflect)]
#[reflect(opaque)]
pub struct PropagationStepRequested;

/// Advance turn (Turn-based mode only)
#[derive(Message, Clone, Reflect)]
#[reflect(opaque)]
pub struct TurnAdvancedMessage;

/// Trigger credibility decay
#[derive(Message, Clone, Reflect)]
#[reflect(opaque)]
pub struct CredibilityDecayRequested {
    pub elapsed_turns: u64,
}

// ==================== State Messages (What Happened) ====================

/// Contagion was spawned
#[derive(Message, Clone, Reflect)]
#[reflect(opaque)]
pub struct ContagionSpawnedEvent {
    pub contagion_entity: Entity,
    pub contagion_id: String,
    pub origin_node: Entity,
}

/// Contagion spread to a new node
#[derive(Message, Clone, Reflect)]
#[reflect(opaque)]
pub struct ContagionSpreadEvent {
    pub infection_entity: Entity,
    pub contagion_id: String,
    pub from_node: Entity,
    pub to_node: Entity,
    pub is_mutation: bool,
    pub original_id: Option<String>,
}

/// Infection state changed (Incubating→Active→Recovered→Plain)
#[derive(Message, Clone, Reflect)]
#[reflect(opaque)]
pub struct InfectionStateChangedEvent {
    pub infection_entity: Entity,
    pub node_entity: Entity,
    pub contagion_id: String,
    pub old_state: InfectionStateType,
    pub new_state: InfectionStateType,
}

/// Infection state type for events
#[derive(Clone, Copy, Reflect, PartialEq, Debug)]
pub enum InfectionStateType {
    Incubating,
    Active,
    Recovered,
    Plain,
}

/// Reinfection occurred
#[derive(Message, Clone, Reflect)]
#[reflect(opaque)]
pub struct ReinfectionOccurredEvent {
    pub infection_entity: Entity,
    pub node_entity: Entity,
    pub contagion_id: String,
}

/// Contagion was removed
#[derive(Message, Clone, Reflect)]
#[reflect(opaque)]
pub struct ContagionRemovedEvent {
    pub contagion_id: String,
    pub reason: RemovalReason,
}

/// Reason for contagion removal
#[derive(Clone, Copy, Reflect, PartialEq, Debug)]
pub enum RemovalReason {
    Expired,
    Manual,
}

/// Propagation step completed
#[derive(Message, Clone, Reflect)]
#[reflect(opaque)]
pub struct PropagationStepCompletedEvent {
    pub spread_count: usize,
    pub mutation_count: usize,
}
