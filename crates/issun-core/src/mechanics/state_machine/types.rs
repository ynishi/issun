//! Core types for state machine mechanics

use crate::mechanics::contagion::{Duration, InfectionStateType};

/// Configuration for state machine behavior
///
/// Defines the duration of each infection stage and whether reinfection is allowed.
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::state_machine::StateMachineConfig;
/// use issun_core::mechanics::contagion::Duration;
///
/// // Short-lived infection with reinfection
/// let config = StateMachineConfig {
///     incubation_duration: Duration::Turns(2),
///     active_duration: Duration::Turns(3),
///     immunity_duration: Duration::Turns(5),
///     allow_reinfection: true,
/// };
///
/// // Permanent immunity
/// let config = StateMachineConfig {
///     incubation_duration: Duration::Turns(3),
///     active_duration: Duration::Turns(5),
///     immunity_duration: Duration::Turns(u64::MAX), // Effectively permanent
///     allow_reinfection: false,
/// };
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct StateMachineConfig {
    /// Duration of incubation period
    pub incubation_duration: Duration,
    /// Duration of active infection
    pub active_duration: Duration,
    /// Duration of immunity after recovery
    pub immunity_duration: Duration,
    /// Whether entities can be reinfected after immunity expires
    pub allow_reinfection: bool,
}

impl Default for StateMachineConfig {
    fn default() -> Self {
        Self {
            incubation_duration: Duration::Turns(3),
            active_duration: Duration::Turns(5),
            immunity_duration: Duration::Turns(10),
            allow_reinfection: true,
        }
    }
}

/// Input for state machine step
///
/// Provides time delta for advancing state timers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StateMachineInput {
    /// Time elapsed since last step
    pub time_delta: Duration,
}

impl StateMachineInput {
    pub fn new(time_delta: Duration) -> Self {
        Self { time_delta }
    }
}

/// Events emitted by state machine
#[derive(Debug, Clone, PartialEq)]
pub enum StateMachineEvent {
    /// Transitioned from one state to another
    StateTransition {
        from: InfectionStateType,
        to: InfectionStateType,
    },

    /// Time advanced in current state
    TimeAdvanced {
        state: InfectionStateType,
        elapsed: Duration,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = StateMachineConfig::default();
        assert_eq!(config.incubation_duration, Duration::Turns(3));
        assert_eq!(config.active_duration, Duration::Turns(5));
        assert_eq!(config.immunity_duration, Duration::Turns(10));
        assert!(config.allow_reinfection);
    }

    #[test]
    fn test_input_new() {
        let input = StateMachineInput::new(Duration::Turns(1));
        assert_eq!(input.time_delta, Duration::Turns(1));
    }
}
