//! Core types for the reputation mechanic.
//!
//! This module defines the fundamental data structures used by the reputation mechanic:
//! - Config: Static configuration (min/max ranges, decay rates)
//! - Input: Per-frame input data (delta changes, elapsed time)
//! - Event: Events emitted when state changes occur
//! - State: Per-entity mutable state (current value)

/// Configuration for the reputation mechanic.
///
/// This type is typically stored as a resource in the game engine and
/// shared across all entities using the reputation mechanic.
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::reputation::ReputationConfig;
///
/// // NPC favorability (0-100)
/// let config = ReputationConfig {
///     min: 0.0,
///     max: 100.0,
///     decay_rate: 0.95, // 5% decay per turn
/// };
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct ReputationConfig {
    /// Minimum allowed value (inclusive)
    pub min: f32,
    /// Maximum allowed value (inclusive)
    pub max: f32,
    /// Decay rate per time unit (0.0 to 1.0, where 1.0 = no decay)
    pub decay_rate: f32,
}

impl Default for ReputationConfig {
    fn default() -> Self {
        Self {
            min: 0.0,
            max: 100.0,
            decay_rate: 1.0, // No decay by default
        }
    }
}

/// Per-frame input for the reputation mechanic.
///
/// This type is constructed fresh each frame from the game world state.
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::reputation::ReputationInput;
///
/// // Player performed a good action, increase reputation by 10
/// let input = ReputationInput {
///     delta: 10.0,
///     elapsed_time: 1,
/// };
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReputationInput {
    /// Change amount (can be positive or negative)
    pub delta: f32,
    /// Time units elapsed since last update (for decay calculation)
    pub elapsed_time: u32,
}

impl Default for ReputationInput {
    fn default() -> Self {
        Self {
            delta: 0.0,
            elapsed_time: 1,
        }
    }
}

/// Per-entity mutable state for the reputation mechanic.
///
/// This type is stored as a component on each entity.
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::reputation::ReputationState;
///
/// let mut state = ReputationState::new(50.0);
/// assert_eq!(state.value, 50.0);
///
/// state.value = 75.0;
/// assert_eq!(state.value, 75.0);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct ReputationState {
    /// Current reputation value
    pub value: f32,
}

impl ReputationState {
    /// Create a new reputation state with the given initial value.
    ///
    /// # Arguments
    ///
    /// * `initial_value` - Starting reputation value
    pub fn new(initial_value: f32) -> Self {
        Self {
            value: initial_value,
        }
    }
}

impl Default for ReputationState {
    fn default() -> Self {
        Self { value: 0.0 }
    }
}

/// Events emitted by the reputation mechanic.
///
/// These events communicate state changes to the game world without
/// coupling the mechanic to any specific engine.
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::reputation::ReputationEvent;
///
/// // Check if value changed
/// match (ReputationEvent::ValueChanged { old_value: 50.0, new_value: 75.0 }) {
///     ReputationEvent::ValueChanged { old_value, new_value } => {
///         println!("Reputation changed from {} to {}", old_value, new_value);
///     }
///     _ => {}
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum ReputationEvent {
    /// Reputation value changed
    ValueChanged {
        /// Previous value
        old_value: f32,
        /// New value (after change and decay)
        new_value: f32,
    },

    /// Reputation reached minimum value
    ReachedMinimum {
        /// The minimum value
        min_value: f32,
    },

    /// Reputation reached maximum value
    ReachedMaximum {
        /// The maximum value
        max_value: f32,
    },

    /// Reputation was clamped to range
    Clamped {
        /// Value before clamping
        attempted_value: f32,
        /// Value after clamping
        clamped_value: f32,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reputation_config_default() {
        let config = ReputationConfig::default();
        assert_eq!(config.min, 0.0);
        assert_eq!(config.max, 100.0);
        assert_eq!(config.decay_rate, 1.0);
    }

    #[test]
    fn test_reputation_input_default() {
        let input = ReputationInput::default();
        assert_eq!(input.delta, 0.0);
        assert_eq!(input.elapsed_time, 1);
    }

    #[test]
    fn test_reputation_state_new() {
        let state = ReputationState::new(42.0);
        assert_eq!(state.value, 42.0);
    }

    #[test]
    fn test_reputation_state_default() {
        let state = ReputationState::default();
        assert_eq!(state.value, 0.0);
    }
}
