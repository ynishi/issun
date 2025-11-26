//! Action Plugin Components
//!
//! Provides per-entity action point management for turn-based game mechanics.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Per-entity action points for turn-based mechanics.
///
/// Any entity (Player, Faction, Group, CPU, etc.) can have ActionPoints.
/// Each entity manages its own action budget independently.
///
/// # Example
///
/// ```rust
/// use bevy::prelude::*;
/// use issun_bevy::plugins::action::ActionPoints;
///
/// fn spawn_player(mut commands: Commands) {
///     commands.spawn((
///         Name::new("Player"),
///         ActionPoints::new(3),
///     ));
/// }
/// ```
#[derive(Component, Clone, Debug, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct ActionPoints {
    /// Current available action points
    pub available: u32,
    /// Maximum action points per period (resets to this value)
    pub max_per_period: u32,
}

impl ActionPoints {
    /// Create new ActionPoints with specified max per period
    pub fn new(max_per_period: u32) -> Self {
        Self {
            available: max_per_period,
            max_per_period,
        }
    }

    /// Consume one action point with context description
    ///
    /// Returns `ActionConsumed` with consumption details, or error if depleted.
    pub fn consume_with(
        &mut self,
        context: impl Into<String>,
    ) -> Result<ActionConsumed, ActionError> {
        if self.available == 0 {
            return Err(ActionError::Depleted);
        }
        self.available -= 1;
        Ok(ActionConsumed {
            context: context.into(),
            remaining: self.available,
            depleted: self.available == 0,
        })
    }

    /// Consume one action point without context
    ///
    /// Returns `true` if successful, `false` if depleted.
    pub fn consume(&mut self) -> bool {
        self.consume_with("").is_ok()
    }

    /// Reset action points to maximum
    pub fn reset(&mut self) {
        self.available = self.max_per_period;
    }

    /// Check if action points are depleted (zero remaining)
    pub fn is_depleted(&self) -> bool {
        self.available == 0
    }

    /// Check if can consume n action points
    pub fn can_consume(&self, n: u32) -> bool {
        self.available >= n
    }
}

impl Default for ActionPoints {
    fn default() -> Self {
        Self::new(3)
    }
}

/// Result of consuming an action point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionConsumed {
    /// Description of the action performed
    pub context: String,
    /// Remaining action points after consumption
    pub remaining: u32,
    /// Whether action points are now depleted (zero remaining)
    pub depleted: bool,
}

/// Error when attempting to consume action points
#[derive(Debug, Clone)]
pub enum ActionError {
    /// Action points are depleted (zero remaining)
    Depleted,
}

impl std::fmt::Display for ActionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActionError::Depleted => write!(f, "Action points depleted"),
        }
    }
}

impl std::error::Error for ActionError {}

/// Global configuration for action points
///
/// Provides default values used when spawning new entities with ActionPoints.
#[derive(Resource, Clone, Debug, Reflect, Serialize, Deserialize)]
#[reflect(Resource)]
pub struct ActionConfig {
    /// Default max actions per period (used when spawning new entities)
    pub default_max_per_period: u32,
}

impl Default for ActionConfig {
    fn default() -> Self {
        Self {
            default_max_per_period: 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_points_new() {
        let points = ActionPoints::new(5);
        assert_eq!(points.available, 5);
        assert_eq!(points.max_per_period, 5);
        assert!(!points.is_depleted());
    }

    #[test]
    fn test_action_points_default() {
        let points = ActionPoints::default();
        assert_eq!(points.available, 3);
        assert_eq!(points.max_per_period, 3);
    }

    #[test]
    fn test_action_points_consume() {
        let mut points = ActionPoints::new(3);

        // Consume with context
        let result = points.consume_with("Deploy troops");
        assert!(result.is_ok());
        let consumed = result.unwrap();
        assert_eq!(consumed.context, "Deploy troops");
        assert_eq!(consumed.remaining, 2);
        assert!(!consumed.depleted);
        assert_eq!(points.available, 2);

        // Consume without context
        assert!(points.consume());
        assert_eq!(points.available, 1);

        // Consume last point
        let result = points.consume_with("Final action");
        assert!(result.is_ok());
        let consumed = result.unwrap();
        assert_eq!(consumed.remaining, 0);
        assert!(consumed.depleted);
        assert!(points.is_depleted());

        // Attempt to consume when depleted
        let result = points.consume_with("Extra action");
        assert!(result.is_err());
        assert!(matches!(result, Err(ActionError::Depleted)));
        assert!(!points.consume());
    }

    #[test]
    fn test_action_points_reset() {
        let mut points = ActionPoints::new(5);
        points.consume();
        points.consume();
        assert_eq!(points.available, 3);

        points.reset();
        assert_eq!(points.available, 5);
        assert!(!points.is_depleted());
    }

    #[test]
    fn test_action_points_can_consume() {
        let points = ActionPoints::new(3);
        assert!(points.can_consume(1));
        assert!(points.can_consume(3));
        assert!(!points.can_consume(4));

        let depleted = ActionPoints::new(0);
        assert!(!depleted.can_consume(1));
    }
}
