//! Infection state machine for advanced contagion mechanics.
//!
//! Provides a 4-stage infection state machine:
//! - Incubating: Infection is present but not yet active
//! - Active: Infection is fully active and most contagious
//! - Recovered: Entity has recovered and has temporary immunity
//! - Plain: Entity is susceptible (no infection, no immunity)

use super::duration::Duration;

/// Four-stage infection state machine.
///
/// Models the full lifecycle of an infection from initial exposure
/// through recovery and potential re-susceptibility.
///
/// # State Transitions
///
/// ```text
/// Plain → Incubating → Active → Recovered → Plain (if reinfection enabled)
///   ↑__________________________________________|
/// ```
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::contagion::{InfectionState, InfectionStateType, Duration};
///
/// // Start in incubating state
/// let state = InfectionState::Incubating {
///     elapsed: Duration::Turns(0),
///     total_duration: Duration::Turns(3),
/// };
///
/// // Check state type
/// assert_eq!(state.state_type(), InfectionStateType::Incubating);
///
/// // Check if it's active (most contagious)
/// assert!(!state.is_active());
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
pub enum InfectionState {
    /// Incubation period - infection present but not yet symptomatic.
    ///
    /// - Low transmission rate
    /// - Progresses to Active when elapsed >= total_duration
    Incubating {
        /// Time elapsed in incubation
        elapsed: Duration,
        /// Total incubation duration
        total_duration: Duration,
    },

    /// Active infection - fully symptomatic and most contagious.
    ///
    /// - High transmission rate
    /// - Progresses to Recovered when elapsed >= total_duration
    Active {
        /// Time elapsed in active state
        elapsed: Duration,
        /// Total active duration
        total_duration: Duration,
    },

    /// Recovered - infection cleared, temporary immunity present.
    ///
    /// - Very low transmission rate (some pathogens linger)
    /// - Progresses to Plain when elapsed >= immunity_duration (if reinfection enabled)
    Recovered {
        /// Time elapsed since recovery
        elapsed: Duration,
        /// Duration of immunity
        immunity_duration: Duration,
    },

    /// Plain - no infection, no immunity, fully susceptible.
    #[default]
    Plain,
}

impl InfectionState {
    /// Get the state type (without duration details).
    ///
    /// # Examples
    ///
    /// ```
    /// use issun_core::mechanics::contagion::{InfectionState, InfectionStateType, Duration};
    ///
    /// let state = InfectionState::Active {
    ///     elapsed: Duration::Turns(2),
    ///     total_duration: Duration::Turns(5),
    /// };
    /// assert_eq!(state.state_type(), InfectionStateType::Active);
    /// ```
    pub fn state_type(&self) -> InfectionStateType {
        match self {
            InfectionState::Incubating { .. } => InfectionStateType::Incubating,
            InfectionState::Active { .. } => InfectionStateType::Active,
            InfectionState::Recovered { .. } => InfectionStateType::Recovered,
            InfectionState::Plain => InfectionStateType::Plain,
        }
    }

    /// Check if the infection is in the active (most contagious) state.
    ///
    /// # Examples
    ///
    /// ```
    /// use issun_core::mechanics::contagion::{InfectionState, Duration};
    ///
    /// let active = InfectionState::Active {
    ///     elapsed: Duration::Turns(1),
    ///     total_duration: Duration::Turns(5),
    /// };
    /// assert!(active.is_active());
    ///
    /// let incubating = InfectionState::Incubating {
    ///     elapsed: Duration::Turns(1),
    ///     total_duration: Duration::Turns(3),
    /// };
    /// assert!(!incubating.is_active());
    /// ```
    pub fn is_active(&self) -> bool {
        matches!(self, InfectionState::Active { .. })
    }

    /// Check if the entity is infected (Incubating or Active).
    ///
    /// # Examples
    ///
    /// ```
    /// use issun_core::mechanics::contagion::{InfectionState, Duration};
    ///
    /// let active = InfectionState::Active {
    ///     elapsed: Duration::Turns(1),
    ///     total_duration: Duration::Turns(5),
    /// };
    /// assert!(active.is_infected());
    ///
    /// let plain = InfectionState::Plain;
    /// assert!(!plain.is_infected());
    /// ```
    pub fn is_infected(&self) -> bool {
        matches!(
            self,
            InfectionState::Incubating { .. } | InfectionState::Active { .. }
        )
    }

    /// Check if the entity has immunity (Recovered state).
    ///
    /// # Examples
    ///
    /// ```
    /// use issun_core::mechanics::contagion::{InfectionState, Duration};
    ///
    /// let recovered = InfectionState::Recovered {
    ///     elapsed: Duration::Turns(1),
    ///     immunity_duration: Duration::Turns(10),
    /// };
    /// assert!(recovered.has_immunity());
    ///
    /// let plain = InfectionState::Plain;
    /// assert!(!plain.has_immunity());
    /// ```
    pub fn has_immunity(&self) -> bool {
        matches!(self, InfectionState::Recovered { .. })
    }

    /// Check if this state should transition to the next state.
    ///
    /// # Examples
    ///
    /// ```
    /// use issun_core::mechanics::contagion::{InfectionState, Duration};
    ///
    /// let mut state = InfectionState::Incubating {
    ///     elapsed: Duration::Turns(3),
    ///     total_duration: Duration::Turns(3),
    /// };
    /// assert!(state.should_transition());
    ///
    /// let mut state = InfectionState::Incubating {
    ///     elapsed: Duration::Turns(2),
    ///     total_duration: Duration::Turns(3),
    /// };
    /// assert!(!state.should_transition());
    /// ```
    pub fn should_transition(&self) -> bool {
        match self {
            InfectionState::Incubating {
                elapsed,
                total_duration,
            } => total_duration.is_expired(elapsed),
            InfectionState::Active {
                elapsed,
                total_duration,
            } => total_duration.is_expired(elapsed),
            InfectionState::Recovered {
                elapsed,
                immunity_duration,
            } => immunity_duration.is_expired(elapsed),
            InfectionState::Plain => false,
        }
    }
}

/// Simplified infection state type (without duration details).
///
/// Used for events and pattern matching where duration details aren't needed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InfectionStateType {
    /// Incubating
    Incubating,
    /// Active
    Active,
    /// Recovered
    Recovered,
    /// Plain (susceptible)
    Plain,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_type() {
        let incubating = InfectionState::Incubating {
            elapsed: Duration::Turns(1),
            total_duration: Duration::Turns(3),
        };
        assert_eq!(incubating.state_type(), InfectionStateType::Incubating);

        let active = InfectionState::Active {
            elapsed: Duration::Turns(2),
            total_duration: Duration::Turns(5),
        };
        assert_eq!(active.state_type(), InfectionStateType::Active);

        let recovered = InfectionState::Recovered {
            elapsed: Duration::Turns(1),
            immunity_duration: Duration::Turns(10),
        };
        assert_eq!(recovered.state_type(), InfectionStateType::Recovered);

        let plain = InfectionState::Plain;
        assert_eq!(plain.state_type(), InfectionStateType::Plain);
    }

    #[test]
    fn test_is_active() {
        let active = InfectionState::Active {
            elapsed: Duration::Turns(1),
            total_duration: Duration::Turns(5),
        };
        assert!(active.is_active());

        let incubating = InfectionState::Incubating {
            elapsed: Duration::Turns(1),
            total_duration: Duration::Turns(3),
        };
        assert!(!incubating.is_active());

        let plain = InfectionState::Plain;
        assert!(!plain.is_active());
    }

    #[test]
    fn test_is_infected() {
        let incubating = InfectionState::Incubating {
            elapsed: Duration::Turns(1),
            total_duration: Duration::Turns(3),
        };
        assert!(incubating.is_infected());

        let active = InfectionState::Active {
            elapsed: Duration::Turns(1),
            total_duration: Duration::Turns(5),
        };
        assert!(active.is_infected());

        let recovered = InfectionState::Recovered {
            elapsed: Duration::Turns(1),
            immunity_duration: Duration::Turns(10),
        };
        assert!(!recovered.is_infected());

        let plain = InfectionState::Plain;
        assert!(!plain.is_infected());
    }

    #[test]
    fn test_has_immunity() {
        let recovered = InfectionState::Recovered {
            elapsed: Duration::Turns(1),
            immunity_duration: Duration::Turns(10),
        };
        assert!(recovered.has_immunity());

        let plain = InfectionState::Plain;
        assert!(!plain.has_immunity());

        let active = InfectionState::Active {
            elapsed: Duration::Turns(1),
            total_duration: Duration::Turns(5),
        };
        assert!(!active.has_immunity());
    }

    #[test]
    fn test_should_transition_incubating() {
        // Not yet ready
        let state = InfectionState::Incubating {
            elapsed: Duration::Turns(2),
            total_duration: Duration::Turns(3),
        };
        assert!(!state.should_transition());

        // Exactly at threshold
        let state = InfectionState::Incubating {
            elapsed: Duration::Turns(3),
            total_duration: Duration::Turns(3),
        };
        assert!(state.should_transition());

        // Past threshold
        let state = InfectionState::Incubating {
            elapsed: Duration::Turns(4),
            total_duration: Duration::Turns(3),
        };
        assert!(state.should_transition());
    }

    #[test]
    fn test_should_transition_active() {
        let state = InfectionState::Active {
            elapsed: Duration::Turns(4),
            total_duration: Duration::Turns(5),
        };
        assert!(!state.should_transition());

        let state = InfectionState::Active {
            elapsed: Duration::Turns(5),
            total_duration: Duration::Turns(5),
        };
        assert!(state.should_transition());
    }

    #[test]
    fn test_should_transition_recovered() {
        let state = InfectionState::Recovered {
            elapsed: Duration::Turns(9),
            immunity_duration: Duration::Turns(10),
        };
        assert!(!state.should_transition());

        let state = InfectionState::Recovered {
            elapsed: Duration::Turns(10),
            immunity_duration: Duration::Turns(10),
        };
        assert!(state.should_transition());
    }

    #[test]
    fn test_should_transition_plain() {
        let state = InfectionState::Plain;
        assert!(!state.should_transition());
    }

    #[test]
    fn test_default() {
        assert_eq!(InfectionState::default(), InfectionState::Plain);
    }
}
