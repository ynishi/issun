//! Time-related resources for game timer management

use serde::{Deserialize, Serialize};

/// Game timer resource for tracking in-game time progression
///
/// This resource provides pure time management without action point coupling.
/// It tracks days and ticks independently of game mechanics.
///
/// # Example
///
/// ```
/// use issun::plugin::GameTimer;
///
/// let mut timer = GameTimer::new();
/// assert_eq!(timer.day, 1);
/// assert_eq!(timer.tick, 0);
///
/// let new_day = timer.increment_day();
/// assert_eq!(new_day, 2);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameTimer {
    /// Current in-game day (starts at 1)
    pub day: u32,
    /// Frame/tick counter for sub-day timing
    pub tick: u64,
}

impl GameTimer {
    /// Create a new game timer starting at day 1
    ///
    /// # Example
    ///
    /// ```
    /// use issun::plugin::GameTimer;
    ///
    /// let timer = GameTimer::new();
    /// assert_eq!(timer.day, 1);
    /// assert_eq!(timer.tick, 0);
    /// ```
    pub fn new() -> Self {
        Self { day: 1, tick: 0 }
    }

    /// Increment day counter
    ///
    /// # Returns
    ///
    /// The new day number
    ///
    /// # Example
    ///
    /// ```
    /// use issun::plugin::GameTimer;
    ///
    /// let mut timer = GameTimer::new();
    /// let new_day = timer.increment_day();
    /// assert_eq!(new_day, 2);
    /// assert_eq!(timer.day, 2);
    /// ```
    pub fn increment_day(&mut self) -> u32 {
        self.day += 1;
        self.day
    }

    /// Increment tick counter (for realtime/sub-day timing)
    ///
    /// # Example
    ///
    /// ```
    /// use issun::plugin::GameTimer;
    ///
    /// let mut timer = GameTimer::new();
    /// timer.tick();
    /// assert_eq!(timer.tick, 1);
    /// ```
    pub fn tick(&mut self) {
        self.tick += 1;
    }

    /// Get current day
    ///
    /// # Example
    ///
    /// ```
    /// use issun::plugin::GameTimer;
    ///
    /// let timer = GameTimer::new();
    /// assert_eq!(timer.current_day(), 1);
    /// ```
    pub fn current_day(&self) -> u32 {
        self.day
    }
}

impl Default for GameTimer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_timer() {
        let timer = GameTimer::new();
        assert_eq!(timer.day, 1);
        assert_eq!(timer.tick, 0);
    }

    #[test]
    fn test_increment_day() {
        let mut timer = GameTimer::new();

        let new_day = timer.increment_day();
        assert_eq!(new_day, 2);
        assert_eq!(timer.day, 2);

        timer.increment_day();
        assert_eq!(timer.day, 3);
    }

    #[test]
    fn test_tick() {
        let mut timer = GameTimer::new();

        timer.tick();
        assert_eq!(timer.tick, 1);

        timer.tick();
        timer.tick();
        assert_eq!(timer.tick, 3);
    }

    #[test]
    fn test_current_day() {
        let timer = GameTimer::new();
        assert_eq!(timer.current_day(), 1);

        let mut timer = GameTimer::new();
        timer.increment_day();
        assert_eq!(timer.current_day(), 2);
    }

    #[test]
    fn test_default() {
        let timer = GameTimer::default();
        assert_eq!(timer.day, 1);
        assert_eq!(timer.tick, 0);
    }

    #[test]
    fn test_independent_day_and_tick() {
        let mut timer = GameTimer::new();

        // Tick doesn't affect day
        timer.tick();
        timer.tick();
        assert_eq!(timer.day, 1);
        assert_eq!(timer.tick, 2);

        // Day doesn't reset tick
        timer.increment_day();
        assert_eq!(timer.day, 2);
        assert_eq!(timer.tick, 2);
    }
}
