//! Action points resource for turn-based game mechanics

use serde::{Deserialize, Serialize};
use std::fmt;

/// Result of successful action consumption
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionConsumed {
    /// What action was performed
    pub context: String,
    /// Actions remaining after consumption
    pub remaining: u32,
    /// Whether all actions are now depleted
    pub depleted: bool,
}

/// Error when trying to consume actions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionError {
    /// No actions remaining
    Depleted,
}

impl fmt::Display for ActionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ActionError::Depleted => write!(f, "No action points remaining"),
        }
    }
}

impl std::error::Error for ActionError {}

/// Action points resource for managing player actions per period
///
/// This resource provides turn-based game mechanics by tracking available
/// action points. It is independent of time management and can be reset
/// by external events (e.g., day changes).
///
/// # Example
///
/// ```
/// use issun::plugin::ActionPoints;
///
/// let mut points = ActionPoints::new(3);
/// assert_eq!(points.available, 3);
///
/// assert!(points.consume());
/// assert_eq!(points.available, 2);
///
/// points.reset();
/// assert_eq!(points.available, 3);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionPoints {
    /// Current available actions
    pub available: u32,
    /// Maximum actions per period (for reset)
    pub max_per_period: u32,
}

impl ActionPoints {
    /// Create a new action points resource
    ///
    /// # Arguments
    ///
    /// * `max_per_period` - Maximum actions available per period
    ///
    /// # Example
    ///
    /// ```
    /// use issun::plugin::ActionPoints;
    ///
    /// let points = ActionPoints::new(5);
    /// assert_eq!(points.available, 5);
    /// assert_eq!(points.max_per_period, 5);
    /// ```
    pub fn new(max_per_period: u32) -> Self {
        Self {
            available: max_per_period,
            max_per_period,
        }
    }

    /// Consume action with context information
    ///
    /// This is the recommended way to consume actions as it provides
    /// context about what action was performed, which can be used for
    /// logging, statistics, and event handling.
    ///
    /// # Arguments
    ///
    /// * `context` - Description of the action being performed
    ///
    /// # Returns
    ///
    /// `Ok(ActionConsumed)` with details if successful, `Err(ActionError)` if depleted
    ///
    /// # Example
    ///
    /// ```
    /// use issun::plugin::action::{ActionPoints, ActionError};
    ///
    /// let mut points = ActionPoints::new(2);
    ///
    /// let result = points.consume_with("Deploy troops");
    /// assert!(result.is_ok());
    /// let consumed = result.unwrap();
    /// assert_eq!(consumed.context, "Deploy troops");
    /// assert_eq!(consumed.remaining, 1);
    /// assert!(!consumed.depleted);
    ///
    /// points.consume_with("Research tech").unwrap();
    /// let result = points.consume_with("Build structure");
    /// assert!(matches!(result, Err(ActionError::Depleted)));
    /// ```
    pub fn consume_with(&mut self, context: impl Into<String>) -> Result<ActionConsumed, ActionError> {
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

    /// Try to consume one action point (without context)
    ///
    /// For simple use cases where context tracking is not needed.
    /// Consider using `consume_with()` for better observability.
    ///
    /// # Returns
    ///
    /// `true` if consumed successfully, `false` if insufficient points
    ///
    /// # Example
    ///
    /// ```
    /// use issun::plugin::ActionPoints;
    ///
    /// let mut points = ActionPoints::new(1);
    /// assert!(points.consume());
    /// assert!(!points.consume()); // No points left
    /// ```
    pub fn consume(&mut self) -> bool {
        self.consume_with("").is_ok()
    }

    /// Try to consume N action points
    ///
    /// # Arguments
    ///
    /// * `n` - Number of points to consume
    ///
    /// # Returns
    ///
    /// `true` if consumed successfully, `false` if insufficient points
    ///
    /// # Example
    ///
    /// ```
    /// use issun::plugin::ActionPoints;
    ///
    /// let mut points = ActionPoints::new(5);
    /// assert!(points.consume_n(3));
    /// assert_eq!(points.available, 2);
    /// assert!(!points.consume_n(3)); // Only 2 left
    /// ```
    pub fn consume_n(&mut self, n: u32) -> bool {
        if self.available >= n {
            self.available -= n;
            true
        } else {
            false
        }
    }

    /// Reset to maximum points (called on period boundary)
    ///
    /// # Example
    ///
    /// ```
    /// use issun::plugin::ActionPoints;
    ///
    /// let mut points = ActionPoints::new(3);
    /// points.consume();
    /// points.consume();
    /// assert_eq!(points.available, 1);
    ///
    /// points.reset();
    /// assert_eq!(points.available, 3);
    /// ```
    pub fn reset(&mut self) {
        self.available = self.max_per_period;
    }

    /// Check if depleted (no actions remaining)
    ///
    /// # Example
    ///
    /// ```
    /// use issun::plugin::ActionPoints;
    ///
    /// let mut points = ActionPoints::new(1);
    /// assert!(!points.is_depleted());
    ///
    /// points.consume();
    /// assert!(points.is_depleted());
    /// ```
    pub fn is_depleted(&self) -> bool {
        self.available == 0
    }

    /// Check if can consume N actions
    ///
    /// # Arguments
    ///
    /// * `n` - Number of points to check
    ///
    /// # Example
    ///
    /// ```
    /// use issun::plugin::ActionPoints;
    ///
    /// let points = ActionPoints::new(3);
    /// assert!(points.can_consume(2));
    /// assert!(points.can_consume(3));
    /// assert!(!points.can_consume(4));
    /// ```
    pub fn can_consume(&self, n: u32) -> bool {
        self.available >= n
    }
}

impl Default for ActionPoints {
    fn default() -> Self {
        Self::new(3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_points() {
        let points = ActionPoints::new(5);
        assert_eq!(points.available, 5);
        assert_eq!(points.max_per_period, 5);
    }

    #[test]
    fn test_consume() {
        let mut points = ActionPoints::new(3);

        assert!(points.consume());
        assert_eq!(points.available, 2);

        assert!(points.consume());
        assert_eq!(points.available, 1);

        assert!(points.consume());
        assert_eq!(points.available, 0);

        assert!(!points.consume());
        assert_eq!(points.available, 0);
    }

    #[test]
    fn test_consume_n() {
        let mut points = ActionPoints::new(5);

        assert!(points.consume_n(2));
        assert_eq!(points.available, 3);

        assert!(points.consume_n(3));
        assert_eq!(points.available, 0);

        assert!(!points.consume_n(1));
    }

    #[test]
    fn test_consume_n_insufficient() {
        let mut points = ActionPoints::new(2);

        assert!(!points.consume_n(3));
        assert_eq!(points.available, 2); // Unchanged
    }

    #[test]
    fn test_reset() {
        let mut points = ActionPoints::new(4);

        points.consume();
        points.consume();
        assert_eq!(points.available, 2);

        points.reset();
        assert_eq!(points.available, 4);
    }

    #[test]
    fn test_is_depleted() {
        let mut points = ActionPoints::new(1);
        assert!(!points.is_depleted());

        points.consume();
        assert!(points.is_depleted());

        points.reset();
        assert!(!points.is_depleted());
    }

    #[test]
    fn test_can_consume() {
        let points = ActionPoints::new(3);

        assert!(points.can_consume(0));
        assert!(points.can_consume(1));
        assert!(points.can_consume(3));
        assert!(!points.can_consume(4));
    }

    #[test]
    fn test_default() {
        let points = ActionPoints::default();
        assert_eq!(points.available, 3);
        assert_eq!(points.max_per_period, 3);
    }

    #[test]
    fn test_consume_with() {
        let mut points = ActionPoints::new(3);

        // First consumption
        let result = points.consume_with("Deploy troops");
        assert!(result.is_ok());
        let consumed = result.unwrap();
        assert_eq!(consumed.context, "Deploy troops");
        assert_eq!(consumed.remaining, 2);
        assert!(!consumed.depleted);

        // Second consumption
        let result = points.consume_with("Research tech");
        assert!(result.is_ok());
        let consumed = result.unwrap();
        assert_eq!(consumed.context, "Research tech");
        assert_eq!(consumed.remaining, 1);
        assert!(!consumed.depleted);

        // Third consumption (depletes)
        let result = points.consume_with("Build structure");
        assert!(result.is_ok());
        let consumed = result.unwrap();
        assert_eq!(consumed.context, "Build structure");
        assert_eq!(consumed.remaining, 0);
        assert!(consumed.depleted);

        // Fourth consumption (should fail)
        let result = points.consume_with("Extra action");
        assert!(result.is_err());
        assert!(matches!(result, Err(ActionError::Depleted)));
    }

    #[test]
    fn test_consume_with_empty_context() {
        let mut points = ActionPoints::new(1);
        let result = points.consume_with("");
        assert!(result.is_ok());
        let consumed = result.unwrap();
        assert_eq!(consumed.context, "");
        assert_eq!(consumed.remaining, 0);
        assert!(consumed.depleted);
    }

    #[test]
    fn test_action_error_display() {
        let error = ActionError::Depleted;
        assert_eq!(error.to_string(), "No action points remaining");
    }
}
