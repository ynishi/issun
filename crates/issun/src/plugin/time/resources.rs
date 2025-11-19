//! Time-related resources for game clock management

use serde::{Deserialize, Serialize};

/// Game clock resource tracking in-game time progression
///
/// This resource holds the current day and remaining actions for turn-based games.
/// It should be mutated by systems that advance time (e.g., when player completes a turn).
///
/// # Example
///
/// ```ignore
/// use issun::plugin::time::GameClock;
///
/// let clock = GameClock::new();
/// assert_eq!(clock.day, 1);
/// assert_eq!(clock.actions_remaining, 3);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameClock {
    /// Current in-game day (starts at 1)
    pub day: u32,
    /// Remaining action points for the current day
    pub actions_remaining: u32,
}

impl GameClock {
    /// Create a new game clock starting at day 1
    ///
    /// # Arguments
    ///
    /// * `actions_per_day` - Number of action points available per day
    pub fn new(actions_per_day: u32) -> Self {
        Self {
            day: 1,
            actions_remaining: actions_per_day,
        }
    }

    /// Advance to the next day and reset action points
    ///
    /// # Arguments
    ///
    /// * `actions_per_day` - Number of action points to reset to
    ///
    /// # Returns
    ///
    /// The new day number
    pub fn advance_day(&mut self, actions_per_day: u32) -> u32 {
        self.day += 1;
        self.actions_remaining = actions_per_day;
        self.day
    }

    /// Consume one action point
    ///
    /// # Returns
    ///
    /// `true` if an action was consumed, `false` if no actions remain
    pub fn consume_action(&mut self) -> bool {
        if self.actions_remaining > 0 {
            self.actions_remaining -= 1;
            true
        } else {
            false
        }
    }

    /// Check if any actions remain for the current day
    pub fn has_actions_remaining(&self) -> bool {
        self.actions_remaining > 0
    }
}

impl Default for GameClock {
    fn default() -> Self {
        Self::new(3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_clock() {
        let clock = GameClock::new(5);
        assert_eq!(clock.day, 1);
        assert_eq!(clock.actions_remaining, 5);
    }

    #[test]
    fn test_advance_day() {
        let mut clock = GameClock::new(3);
        clock.actions_remaining = 0;

        let new_day = clock.advance_day(3);
        assert_eq!(new_day, 2);
        assert_eq!(clock.day, 2);
        assert_eq!(clock.actions_remaining, 3);
    }

    #[test]
    fn test_consume_action() {
        let mut clock = GameClock::new(2);
        assert!(clock.consume_action());
        assert_eq!(clock.actions_remaining, 1);

        assert!(clock.consume_action());
        assert_eq!(clock.actions_remaining, 0);

        assert!(!clock.consume_action());
        assert_eq!(clock.actions_remaining, 0);
    }

    #[test]
    fn test_has_actions_remaining() {
        let mut clock = GameClock::new(1);
        assert!(clock.has_actions_remaining());

        clock.consume_action();
        assert!(!clock.has_actions_remaining());
    }

    #[test]
    fn test_default() {
        let clock = GameClock::default();
        assert_eq!(clock.day, 1);
        assert_eq!(clock.actions_remaining, 3);
    }
}
