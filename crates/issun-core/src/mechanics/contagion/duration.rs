//! Time duration abstraction for contagion mechanics.
//!
//! Supports three different time modes:
//! - Turn-based: Discrete turns (e.g., strategy games)
//! - Tick-based: Frame-based ticks (e.g., simulation games)
//! - Time-based: Real-time seconds (e.g., action games)

/// Time duration abstraction that supports multiple time modes.
///
/// This enum allows contagion mechanics to work across different game types
/// without coupling to a specific time representation.
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::contagion::Duration;
///
/// // Turn-based game
/// let turn_duration = Duration::Turns(5);
///
/// // Tick-based game
/// let tick_duration = Duration::Ticks(100);
///
/// // Time-based game
/// let time_duration = Duration::Seconds(3.5);
///
/// // Check if duration has expired
/// let elapsed = Duration::Turns(6);
/// assert!(turn_duration.is_expired(&elapsed));
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Duration {
    /// Discrete turns (strategy games, board games)
    Turns(u64),
    /// Frame-based ticks (simulations, fixed timestep)
    Ticks(u64),
    /// Real-time seconds (action games, variable timestep)
    Seconds(f32),
}

impl Duration {
    /// Create a zero duration for the given time mode.
    ///
    /// # Examples
    ///
    /// ```
    /// use issun_core::mechanics::contagion::Duration;
    ///
    /// let zero_turns = Duration::zero_turns();
    /// assert_eq!(zero_turns, Duration::Turns(0));
    ///
    /// let zero_ticks = Duration::zero_ticks();
    /// assert_eq!(zero_ticks, Duration::Ticks(0));
    ///
    /// let zero_seconds = Duration::zero_seconds();
    /// assert_eq!(zero_seconds, Duration::Seconds(0.0));
    /// ```
    pub fn zero_turns() -> Self {
        Duration::Turns(0)
    }

    /// Create a zero duration in ticks.
    pub fn zero_ticks() -> Self {
        Duration::Ticks(0)
    }

    /// Create a zero duration in seconds.
    pub fn zero_seconds() -> Self {
        Duration::Seconds(0.0)
    }

    /// Check if this duration has expired compared to elapsed time.
    ///
    /// Returns `true` if `elapsed >= self`, `false` otherwise.
    /// If the time modes don't match, returns `false`.
    ///
    /// # Examples
    ///
    /// ```
    /// use issun_core::mechanics::contagion::Duration;
    ///
    /// let total = Duration::Turns(5);
    /// let elapsed = Duration::Turns(6);
    /// assert!(total.is_expired(&elapsed));
    ///
    /// let not_yet = Duration::Turns(4);
    /// assert!(!total.is_expired(&not_yet));
    ///
    /// // Mismatched modes return false
    /// let wrong_mode = Duration::Ticks(10);
    /// assert!(!total.is_expired(&wrong_mode));
    /// ```
    pub fn is_expired(&self, elapsed: &Duration) -> bool {
        match (self, elapsed) {
            (Duration::Turns(total), Duration::Turns(e)) => e >= total,
            (Duration::Ticks(total), Duration::Ticks(e)) => e >= total,
            (Duration::Seconds(total), Duration::Seconds(e)) => e >= total,
            _ => false, // Mismatched modes
        }
    }

    /// Add another duration to this one (mutable).
    ///
    /// Only adds if the time modes match. Otherwise, does nothing.
    ///
    /// # Examples
    ///
    /// ```
    /// use issun_core::mechanics::contagion::Duration;
    ///
    /// let mut duration = Duration::Turns(5);
    /// duration.add(&Duration::Turns(3));
    /// assert_eq!(duration, Duration::Turns(8));
    ///
    /// // Mismatched modes are ignored
    /// duration.add(&Duration::Ticks(10));
    /// assert_eq!(duration, Duration::Turns(8)); // Unchanged
    /// ```
    pub fn add(&mut self, delta: &Duration) {
        match (self, delta) {
            (Duration::Turns(ref mut e), Duration::Turns(d)) => *e += d,
            (Duration::Ticks(ref mut e), Duration::Ticks(d)) => *e += d,
            (Duration::Seconds(ref mut e), Duration::Seconds(d)) => *e += d,
            _ => {} // Mismatched modes, do nothing
        }
    }

    /// Get the raw value as a float (for generic operations).
    ///
    /// # Examples
    ///
    /// ```
    /// use issun_core::mechanics::contagion::Duration;
    ///
    /// assert_eq!(Duration::Turns(5).as_f32(), 5.0);
    /// assert_eq!(Duration::Ticks(100).as_f32(), 100.0);
    /// assert_eq!(Duration::Seconds(3.5).as_f32(), 3.5);
    /// ```
    pub fn as_f32(&self) -> f32 {
        match self {
            Duration::Turns(t) => *t as f32,
            Duration::Ticks(t) => *t as f32,
            Duration::Seconds(s) => *s,
        }
    }
}

impl Default for Duration {
    fn default() -> Self {
        Duration::Turns(0)
    }
}

impl std::ops::Add for Duration {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Duration::Turns(a), Duration::Turns(b)) => Duration::Turns(a + b),
            (Duration::Ticks(a), Duration::Ticks(b)) => Duration::Ticks(a + b),
            (Duration::Seconds(a), Duration::Seconds(b)) => Duration::Seconds(a + b),
            _ => self, // Mismatched modes, return left operand
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_constructors() {
        assert_eq!(Duration::zero_turns(), Duration::Turns(0));
        assert_eq!(Duration::zero_ticks(), Duration::Ticks(0));
        assert_eq!(Duration::zero_seconds(), Duration::Seconds(0.0));
    }

    #[test]
    fn test_is_expired_turns() {
        let total = Duration::Turns(5);
        assert!(!total.is_expired(&Duration::Turns(4)));
        assert!(total.is_expired(&Duration::Turns(5)));
        assert!(total.is_expired(&Duration::Turns(6)));
    }

    #[test]
    fn test_is_expired_ticks() {
        let total = Duration::Ticks(100);
        assert!(!total.is_expired(&Duration::Ticks(99)));
        assert!(total.is_expired(&Duration::Ticks(100)));
        assert!(total.is_expired(&Duration::Ticks(101)));
    }

    #[test]
    fn test_is_expired_seconds() {
        let total = Duration::Seconds(3.5);
        assert!(!total.is_expired(&Duration::Seconds(3.4)));
        assert!(total.is_expired(&Duration::Seconds(3.5)));
        assert!(total.is_expired(&Duration::Seconds(3.6)));
    }

    #[test]
    fn test_is_expired_mismatched_modes() {
        let total = Duration::Turns(5);
        assert!(!total.is_expired(&Duration::Ticks(10)));
        assert!(!total.is_expired(&Duration::Seconds(10.0)));
    }

    #[test]
    fn test_add_same_mode() {
        let mut duration = Duration::Turns(5);
        duration.add(&Duration::Turns(3));
        assert_eq!(duration, Duration::Turns(8));

        let mut duration = Duration::Ticks(100);
        duration.add(&Duration::Ticks(50));
        assert_eq!(duration, Duration::Ticks(150));

        let mut duration = Duration::Seconds(2.5);
        duration.add(&Duration::Seconds(1.5));
        assert!((duration.as_f32() - 4.0).abs() < 0.001);
    }

    #[test]
    fn test_add_mismatched_mode() {
        let mut duration = Duration::Turns(5);
        duration.add(&Duration::Ticks(10));
        assert_eq!(duration, Duration::Turns(5)); // Unchanged

        duration.add(&Duration::Seconds(3.0));
        assert_eq!(duration, Duration::Turns(5)); // Unchanged
    }

    #[test]
    fn test_as_f32() {
        assert_eq!(Duration::Turns(5).as_f32(), 5.0);
        assert_eq!(Duration::Ticks(100).as_f32(), 100.0);
        assert_eq!(Duration::Seconds(3.5).as_f32(), 3.5);
    }
}
