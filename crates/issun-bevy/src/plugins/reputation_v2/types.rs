//! Bevy-specific types and adapters for reputation_v2 plugin.
//!
//! This module provides the glue between issun-core's pure reputation logic
//! and Bevy's ECS system.

use bevy::{ecs::message::MessageWriter, prelude::*};
use issun_core::mechanics::reputation::{ReputationConfig, ReputationEvent, ReputationState};
use issun_core::mechanics::EventEmitter;

/// Reputation configuration resource - wraps issun-core's ReputationConfig
#[derive(Resource, Clone, Reflect)]
#[reflect(Resource)]
#[derive(Default)]
pub struct ReputationConfigResource {
    #[reflect(ignore)]
    pub config: ReputationConfig,
}

impl ReputationConfigResource {
    pub fn new(config: ReputationConfig) -> Self {
        Self { config }
    }
}

/// Message wrapper for issun-core's ReputationEvent (Bevy 0.17+)
#[derive(bevy::ecs::message::Message, Clone, Debug)]
pub struct ReputationEventWrapper {
    pub entity: Entity,
    pub event: ReputationEvent,
}

/// Bevy adapter: Wraps Bevy's MessageWriter to implement EventEmitter.
///
/// This allows issun-core's reputation mechanic to emit events into Bevy's
/// message system without depending on Bevy directly.
pub struct BevyEventEmitter<'a, 'b> {
    entity: Entity,
    writer: &'a mut MessageWriter<'b, ReputationEventWrapper>,
}

impl<'a, 'b> BevyEventEmitter<'a, 'b> {
    /// Create a new Bevy event emitter.
    pub fn new(entity: Entity, writer: &'a mut MessageWriter<'b, ReputationEventWrapper>) -> Self {
        Self { entity, writer }
    }
}

impl<'a, 'b> EventEmitter<ReputationEvent> for BevyEventEmitter<'a, 'b> {
    fn emit(&mut self, event: ReputationEvent) {
        self.writer.write(ReputationEventWrapper {
            entity: self.entity,
            event,
        });
    }
}

/// Component: Reputation value (maps to ReputationState internally).
///
/// This component stores the reputation state (value) for an entity.
/// It wraps issun-core's ReputationState but provides Bevy-specific functionality.
#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct ReputationValue {
    /// Current reputation value
    pub value: f32,
}

impl ReputationValue {
    /// Create a new ReputationValue component with the given initial value.
    pub fn new(initial_value: f32) -> Self {
        Self {
            value: initial_value,
        }
    }

    /// Convert to issun-core's ReputationState.
    pub fn to_reputation_state(&self) -> ReputationState {
        ReputationState { value: self.value }
    }

    /// Update from issun-core's ReputationState.
    pub fn from_reputation_state(&mut self, state: &ReputationState) {
        self.value = state.value;
    }
}

impl Default for ReputationValue {
    fn default() -> Self {
        Self { value: 0.0 }
    }
}

/// Message: Request reputation change.
#[derive(bevy::ecs::message::Message, Clone, Debug)]
pub struct ReputationChangeRequested {
    /// Entity whose reputation should change
    pub entity: Entity,

    /// Change amount (can be positive or negative)
    pub delta: f32,

    /// Time units elapsed since last update (for decay calculation)
    pub elapsed_time: u32,
}

/// Message: Reputation value changed.
#[derive(bevy::ecs::message::Message, Clone, Debug)]
pub struct ReputationChanged {
    /// Entity whose reputation changed
    pub entity: Entity,

    /// Old value
    pub old_value: f32,

    /// New value
    pub new_value: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reputation_value_component() {
        let reputation = ReputationValue::new(50.0);
        assert_eq!(reputation.value, 50.0);
    }

    #[test]
    fn test_reputation_value_default() {
        let reputation = ReputationValue::default();
        assert_eq!(reputation.value, 0.0);
    }

    #[test]
    fn test_reputation_value_to_state() {
        let reputation = ReputationValue { value: 75.5 };
        let state = reputation.to_reputation_state();
        assert_eq!(state.value, 75.5);
    }

    #[test]
    fn test_reputation_value_from_state() {
        let mut reputation = ReputationValue::new(50.0);
        let state = ReputationState { value: 80.0 };
        reputation.from_reputation_state(&state);
        assert_eq!(reputation.value, 80.0);
    }
}
