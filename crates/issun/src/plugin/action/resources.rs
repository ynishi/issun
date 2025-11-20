//! Action points resource for turn-based game mechanics

use serde::{Deserialize, Serialize};

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

    /// Try to consume one action point
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
        if self.available > 0 {
            self.available -= 1;
            true
        } else {
            false
        }
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
}
