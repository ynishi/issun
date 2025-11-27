//! Type definitions for the Evolution mechanic.
//!
//! This module defines the core types used by the Natural Evolution Mechanic,
//! which models time-based natural state changes (growth, decay, oscillation).

use std::collections::HashMap;

/// Configuration for evolution mechanics (shared across entities).
#[derive(Debug, Clone)]
pub struct EvolutionConfig {
    /// Base rate of evolution per unit time
    pub base_rate: f32,

    /// Default time delta (can be overridden by input)
    pub time_delta: f32,
}

impl Default for EvolutionConfig {
    fn default() -> Self {
        Self {
            base_rate: 1.0,
            time_delta: 1.0,
        }
    }
}

/// Per-entity evolution state.
#[derive(Debug, Clone)]
pub struct EvolutionState {
    /// Current value (e.g., food freshness 0-100, plant size, etc.)
    pub value: f32,

    /// Minimum bound (value cannot go below this)
    pub min: f32,

    /// Maximum bound (value cannot exceed this)
    pub max: f32,

    /// Custom rate multiplier for this specific entity
    pub rate_multiplier: f32,

    /// What type of subject is evolving
    pub subject: SubjectType,

    /// Current evolution status
    pub status: EvolutionStatus,
}

impl Default for EvolutionState {
    fn default() -> Self {
        Self {
            value: 0.0,
            min: 0.0,
            max: 100.0,
            rate_multiplier: 1.0,
            subject: SubjectType::Custom(0),
            status: EvolutionStatus::Active,
        }
    }
}

impl EvolutionState {
    /// Create a new evolution state with specified bounds
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

/// Input for a single evolution step.
#[derive(Debug, Clone)]
pub struct EvolutionInput {
    /// Time elapsed since last update
    pub time_delta: f32,

    /// Environmental conditions affecting this entity
    pub environment: Environment,
}

impl Default for EvolutionInput {
    fn default() -> Self {
        Self {
            time_delta: 1.0,
            environment: Environment::default(),
        }
    }
}

/// Events emitted during evolution.
#[derive(Debug, Clone, PartialEq)]
pub enum EvolutionEvent {
    /// Value changed
    ValueChanged {
        old_value: f32,
        new_value: f32,
        delta: f32,
    },

    /// Reached minimum bound
    MinimumReached { final_value: f32 },

    /// Reached maximum bound
    MaximumReached { final_value: f32 },

    /// Crossed a threshold
    ThresholdCrossed {
        threshold: f32,
        direction: Direction,
    },

    /// Status changed
    StatusChanged {
        old_status: EvolutionStatus,
        new_status: EvolutionStatus,
    },
}

/// Type of subject undergoing evolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SubjectType {
    /// Food items (spoilage, decay)
    Food,

    /// Plants (growth)
    Plant,

    /// Natural resources (regeneration, depletion)
    Resource,

    /// Equipment (degradation, wear)
    Equipment,

    /// Population (growth, decline)
    Population,

    /// Custom type with numeric ID
    Custom(u32),
}

/// Current status of evolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EvolutionStatus {
    /// Evolution is actively occurring
    Active,

    /// Evolution is temporarily paused
    Paused,

    /// Evolution has reached its completion state
    Completed,

    /// Entity is depleted (value at minimum)
    Depleted,
}

/// Environmental conditions that can affect evolution.
#[derive(Debug, Clone)]
pub struct Environment {
    /// Temperature in Celsius
    pub temperature: f32,

    /// Humidity (0.0 = 0%, 1.0 = 100%)
    pub humidity: f32,

    /// Atmospheric pressure (arbitrary units, 1.0 = normal)
    pub pressure: f32,

    /// Custom environmental factors
    pub custom: HashMap<String, f32>,
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            temperature: 20.0,  // Room temperature
            humidity: 0.5,      // 50% humidity
            pressure: 1.0,      // Normal pressure
            custom: HashMap::new(),
        }
    }
}

impl Environment {
    /// Create environment with specified temperature and humidity
    pub fn new(temperature: f32, humidity: f32) -> Self {
        Self {
            temperature,
            humidity,
            pressure: 1.0,
            custom: HashMap::new(),
        }
    }

    /// Add a custom environmental factor
    pub fn with_custom(mut self, key: impl Into<String>, value: f32) -> Self {
        self.custom.insert(key.into(), value);
        self
    }

    /// Get a custom environmental factor
    pub fn get_custom(&self, key: &str) -> Option<f32> {
        self.custom.get(key).copied()
    }
}

/// Direction of value change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    /// Value is increasing
    Increasing,

    /// Value is decreasing
    Decreasing,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evolution_state_creation() {
        let state = EvolutionState::new(50.0, 0.0, 100.0, SubjectType::Food);
        assert_eq!(state.value, 50.0);
        assert_eq!(state.min, 0.0);
        assert_eq!(state.max, 100.0);
        assert_eq!(state.subject, SubjectType::Food);
    }

    #[test]
    fn test_evolution_state_clamping() {
        let state = EvolutionState::new(150.0, 0.0, 100.0, SubjectType::Plant);
        assert_eq!(state.value, 100.0); // Clamped to max
    }

    #[test]
    fn test_normalized_value() {
        let state = EvolutionState::new(50.0, 0.0, 100.0, SubjectType::Resource);
        assert!((state.normalized() - 0.5).abs() < f32::EPSILON);

        let state2 = EvolutionState::new(25.0, 0.0, 100.0, SubjectType::Resource);
        assert!((state2.normalized() - 0.25).abs() < f32::EPSILON);
    }

    #[test]
    fn test_is_at_bounds() {
        let mut state = EvolutionState::new(0.0, 0.0, 100.0, SubjectType::Food);
        assert!(state.is_at_min());
        assert!(!state.is_at_max());

        state.value = 100.0;
        assert!(!state.is_at_min());
        assert!(state.is_at_max());
    }

    #[test]
    fn test_environment_custom_values() {
        let env = Environment::default()
            .with_custom("light", 0.8)
            .with_custom("nutrients", 0.6);

        assert_eq!(env.get_custom("light"), Some(0.8));
        assert_eq!(env.get_custom("nutrients"), Some(0.6));
        assert_eq!(env.get_custom("unknown"), None);
    }

    #[test]
    fn test_evolution_status() {
        let mut state = EvolutionState::default();
        assert!(state.is_active());

        state.status = EvolutionStatus::Paused;
        assert!(!state.is_active());

        state.status = EvolutionStatus::Completed;
        assert!(!state.is_active());
    }
}
