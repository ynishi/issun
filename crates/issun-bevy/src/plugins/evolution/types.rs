//! Bevy-specific types and adapters for evolution plugin.
//!
//! This module provides the glue between issun-core's pure evolution logic
//! and Bevy's ECS system.

use bevy::{ecs::message::MessageWriter, prelude::*};
use issun_core::mechanics::evolution::{
    EvolutionConfig, EvolutionEvent, SubjectType as CoreSubjectType,
};
use issun_core::mechanics::EventEmitter;

/// Evolution configuration resource - wraps issun-core's EvolutionConfig
#[derive(Resource, Clone, Reflect, Default)]
#[reflect(Resource)]
pub struct EvolutionConfigResource {
    #[reflect(ignore)]
    pub config: EvolutionConfig,
}

impl EvolutionConfigResource {
    pub fn new(config: EvolutionConfig) -> Self {
        Self { config }
    }
}

/// Component: Evolution state (wraps issun-core's EvolutionState).
///
/// This component stores the evolution state for an entity.
/// It wraps issun-core's EvolutionState but provides Bevy-specific functionality.
#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct EvolutionStateComponent {
    /// Current value (e.g., food freshness 0-100, plant size)
    pub value: f32,

    /// Minimum bound
    pub min: f32,

    /// Maximum bound
    pub max: f32,

    /// Custom rate multiplier for this entity
    pub rate_multiplier: f32,

    /// What type of subject is evolving
    pub subject: SubjectType,

    /// Current evolution status
    pub status: EvolutionStatus,
}

impl EvolutionStateComponent {
    /// Create a new evolution state component
    pub fn new(initial_value: f32, min: f32, max: f32, subject: SubjectType) -> Self {
        Self {
            value: initial_value.clamp(min, max),
            min,
            max,
            rate_multiplier: 1.0,
            subject,
            status: EvolutionStatus::Active,
        }
    }

    /// Convert to issun-core's EvolutionState
    pub fn to_evolution_state(&self) -> issun_core::mechanics::evolution::EvolutionState {
        issun_core::mechanics::evolution::EvolutionState {
            value: self.value,
            min: self.min,
            max: self.max,
            rate_multiplier: self.rate_multiplier,
            subject: self.subject.to_core_subject_type(),
            status: self.status.to_core_status(),
        }
    }

    /// Update from issun-core's EvolutionState
    pub fn from_evolution_state(
        &mut self,
        state: &issun_core::mechanics::evolution::EvolutionState,
    ) {
        self.value = state.value;
        self.min = state.min;
        self.max = state.max;
        self.rate_multiplier = state.rate_multiplier;
        self.subject = SubjectType::from_core_subject_type(state.subject);
        self.status = EvolutionStatus::from_core_status(state.status);
    }

    /// Check if value is at minimum
    pub fn is_at_min(&self) -> bool {
        (self.value - self.min).abs() < f32::EPSILON
    }

    /// Check if value is at maximum
    pub fn is_at_max(&self) -> bool {
        (self.value - self.max).abs() < f32::EPSILON
    }

    /// Get normalized value (0.0 to 1.0)
    pub fn normalized(&self) -> f32 {
        if (self.max - self.min).abs() < f32::EPSILON {
            0.0
        } else {
            (self.value - self.min) / (self.max - self.min)
        }
    }

    /// Check if evolution is active
    pub fn is_active(&self) -> bool {
        matches!(self.status, EvolutionStatus::Active)
    }
}

/// Type of subject undergoing evolution (Bevy version)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum SubjectType {
    Food,
    Plant,
    Resource,
    Equipment,
    Population,
    Custom(u32),
}

impl SubjectType {
    fn to_core_subject_type(self) -> CoreSubjectType {
        match self {
            SubjectType::Food => CoreSubjectType::Food,
            SubjectType::Plant => CoreSubjectType::Plant,
            SubjectType::Resource => CoreSubjectType::Resource,
            SubjectType::Equipment => CoreSubjectType::Equipment,
            SubjectType::Population => CoreSubjectType::Population,
            SubjectType::Custom(id) => CoreSubjectType::Custom(id),
        }
    }

    fn from_core_subject_type(core: CoreSubjectType) -> Self {
        match core {
            CoreSubjectType::Food => SubjectType::Food,
            CoreSubjectType::Plant => SubjectType::Plant,
            CoreSubjectType::Resource => SubjectType::Resource,
            CoreSubjectType::Equipment => SubjectType::Equipment,
            CoreSubjectType::Population => SubjectType::Population,
            CoreSubjectType::Custom(id) => SubjectType::Custom(id),
        }
    }
}

/// Current status of evolution (Bevy version)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum EvolutionStatus {
    Active,
    Paused,
    Completed,
    Depleted,
}

impl EvolutionStatus {
    fn to_core_status(self) -> issun_core::mechanics::evolution::EvolutionStatus {
        match self {
            EvolutionStatus::Active => issun_core::mechanics::evolution::EvolutionStatus::Active,
            EvolutionStatus::Paused => issun_core::mechanics::evolution::EvolutionStatus::Paused,
            EvolutionStatus::Completed => {
                issun_core::mechanics::evolution::EvolutionStatus::Completed
            }
            EvolutionStatus::Depleted => {
                issun_core::mechanics::evolution::EvolutionStatus::Depleted
            }
        }
    }

    fn from_core_status(core: issun_core::mechanics::evolution::EvolutionStatus) -> Self {
        match core {
            issun_core::mechanics::evolution::EvolutionStatus::Active => EvolutionStatus::Active,
            issun_core::mechanics::evolution::EvolutionStatus::Paused => EvolutionStatus::Paused,
            issun_core::mechanics::evolution::EvolutionStatus::Completed => {
                EvolutionStatus::Completed
            }
            issun_core::mechanics::evolution::EvolutionStatus::Depleted => {
                EvolutionStatus::Depleted
            }
        }
    }
}

/// Component: Environment conditions affecting this entity (wraps issun-core's Environment).
#[derive(Debug, Component, Clone, Reflect)]
#[reflect(Component)]
pub struct EnvironmentComponent {
    /// Temperature in Celsius
    pub temperature: f32,

    /// Humidity (0.0 = 0%, 1.0 = 100%)
    pub humidity: f32,

    /// Atmospheric pressure (1.0 = normal)
    pub pressure: f32,
}

impl Default for EnvironmentComponent {
    fn default() -> Self {
        Self {
            temperature: 20.0,
            humidity: 0.5,
            pressure: 1.0,
        }
    }
}

impl EnvironmentComponent {
    /// Create new environment
    pub fn new(temperature: f32, humidity: f32) -> Self {
        Self {
            temperature,
            humidity,
            pressure: 1.0,
        }
    }

    /// Convert to issun-core's Environment
    pub fn to_core_environment(&self) -> issun_core::mechanics::evolution::Environment {
        issun_core::mechanics::evolution::Environment {
            temperature: self.temperature,
            humidity: self.humidity,
            pressure: self.pressure,
            custom: std::collections::HashMap::new(),
        }
    }
}

/// Message wrapper for issun-core's EvolutionEvent (Bevy 0.17+)
#[derive(bevy::ecs::message::Message, Clone, Debug, PartialEq)]
pub struct EvolutionEventWrapper {
    pub entity: Entity,
    pub event: EvolutionEvent,
}

/// Bevy adapter: Wraps Bevy's MessageWriter to implement EventEmitter.
///
/// This allows issun-core's evolution mechanic to emit events into Bevy's
/// message system without depending on Bevy directly.
pub struct BevyEventEmitter<'a, 'b> {
    entity: Entity,
    writer: &'a mut MessageWriter<'b, EvolutionEventWrapper>,
}

impl<'a, 'b> BevyEventEmitter<'a, 'b> {
    /// Create a new Bevy event emitter.
    pub fn new(entity: Entity, writer: &'a mut MessageWriter<'b, EvolutionEventWrapper>) -> Self {
        Self { entity, writer }
    }
}

impl<'a, 'b> EventEmitter<EvolutionEvent> for BevyEventEmitter<'a, 'b> {
    fn emit(&mut self, event: EvolutionEvent) {
        self.writer.write(EvolutionEventWrapper {
            entity: self.entity,
            event,
        });
    }
}

/// Message: Request evolution tick for an entity.
#[derive(bevy::ecs::message::Message, Clone, Debug)]
pub struct EvolutionTick {
    /// Entity to evolve
    pub entity: Entity,

    /// Custom time delta (if None, uses config default)
    pub time_delta: Option<f32>,

    /// Custom environment (if None, uses EnvironmentComponent or default)
    pub custom_environment: Option<EnvironmentComponent>,
}

impl EvolutionTick {
    /// Create a simple tick with default settings
    pub fn new(entity: Entity) -> Self {
        Self {
            entity,
            time_delta: None,
            custom_environment: None,
        }
    }

    /// Create a tick with custom time delta
    pub fn with_time_delta(entity: Entity, time_delta: f32) -> Self {
        Self {
            entity,
            time_delta: Some(time_delta),
            custom_environment: None,
        }
    }

    /// Create a tick with custom environment
    pub fn with_environment(entity: Entity, environment: EnvironmentComponent) -> Self {
        Self {
            entity,
            time_delta: None,
            custom_environment: Some(environment),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evolution_state_component() {
        let state = EvolutionStateComponent::new(50.0, 0.0, 100.0, SubjectType::Food);
        assert_eq!(state.value, 50.0);
        assert_eq!(state.min, 0.0);
        assert_eq!(state.max, 100.0);
        assert!(state.is_active());
    }

    #[test]
    fn test_evolution_state_to_core() {
        let bevy_state = EvolutionStateComponent::new(75.0, 0.0, 100.0, SubjectType::Plant);
        let core_state = bevy_state.to_evolution_state();
        assert_eq!(core_state.value, 75.0);
    }

    #[test]
    fn test_evolution_state_from_core() {
        let mut bevy_state = EvolutionStateComponent::new(50.0, 0.0, 100.0, SubjectType::Food);
        let core_state = issun_core::mechanics::evolution::EvolutionState {
            value: 25.0,
            min: 0.0,
            max: 100.0,
            rate_multiplier: 1.0,
            subject: CoreSubjectType::Food,
            status: issun_core::mechanics::evolution::EvolutionStatus::Active,
        };
        bevy_state.from_evolution_state(&core_state);
        assert_eq!(bevy_state.value, 25.0);
    }

    #[test]
    fn test_environment_component() {
        let env = EnvironmentComponent::new(25.0, 0.7);
        assert_eq!(env.temperature, 25.0);
        assert_eq!(env.humidity, 0.7);
    }

    #[test]
    fn test_evolution_tick() {
        let entity = Entity::PLACEHOLDER;
        let tick = EvolutionTick::new(entity);
        assert_eq!(tick.entity, entity);
        assert!(tick.time_delta.is_none());
    }
}
