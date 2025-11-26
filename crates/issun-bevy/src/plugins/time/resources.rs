//! Time-related resources

use bevy::prelude::*;

use super::states::TurnPhase;

/// Game date resource tracking in-game time
///
/// Provides day counter and tick counter for time-based mechanics.
/// Does NOT couple with action points (see ActionPlugin for that).
///
/// **Naming**: Uses "Date" instead of "Timer" to avoid confusion with Bevy's `Time` resource.
///
/// # Example
///
/// ```ignore
/// use issun_bevy::plugins::time::GameDate;
///
/// let mut date = GameDate::new();
/// assert_eq!(date.day, 1);
/// assert_eq!(date.tick, 0);
///
/// let new_day = date.increment_day();
/// assert_eq!(new_day, 2);
/// ```
#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct GameDate {
    /// Current in-game day (starts at 1)
    pub day: u32,

    /// Frame/tick counter for sub-day timing
    pub tick: u64,
}

impl GameDate {
    /// Create a new game date starting at day 1
    pub fn new() -> Self {
        Self { day: 1, tick: 0 }
    }

    /// Increment day counter
    ///
    /// # Returns
    ///
    /// The new day number
    pub fn increment_day(&mut self) -> u32 {
        self.day += 1;
        self.day
    }

    /// Increment tick counter (for realtime/sub-day timing)
    pub fn tick(&mut self) {
        self.tick += 1;
    }

    /// Get current day
    pub fn current_day(&self) -> u32 {
        self.day
    }
}

impl Default for GameDate {
    fn default() -> Self {
        Self::new()
    }
}

/// Resource for reserving the next turn phase
///
/// Allows logic systems to "book" the next phase transition target,
/// which will be applied when AnimationLock count reaches zero.
///
/// # Example
///
/// ```ignore
/// // Logic system reserves enemy turn
/// fn end_player_turn(mut next_phase: ResMut<NextTurnPhase>) {
///     next_phase.reserve(TurnPhase::EnemyTurn);
/// }
///
/// // Visual lock release system applies the reservation
/// fn check_animations_done(
///     locks: Query<&AnimationLock>,
///     next_phase: Res<NextTurnPhase>,
///     mut state: ResMut<NextState<TurnPhase>>,
/// ) {
///     if locks.is_empty() {
///         if let Some(phase) = next_phase.get() {
///             state.set(phase);
///         }
///     }
/// }
/// ```
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct NextTurnPhase {
    reserved: Option<TurnPhase>,
}

impl NextTurnPhase {
    /// Reserve a phase transition target
    pub fn reserve(&mut self, phase: TurnPhase) {
        self.reserved = Some(phase);
    }

    /// Get the reserved phase (if any)
    pub fn get(&self) -> Option<TurnPhase> {
        self.reserved.clone()
    }

    /// Clear the reservation
    pub fn clear(&mut self) {
        self.reserved = None;
    }

    /// Check if a phase is reserved
    pub fn is_reserved(&self) -> bool {
        self.reserved.is_some()
    }
}

/// Time plugin configuration
#[derive(Resource, Clone, Reflect)]
#[reflect(Resource)]
pub struct TimeConfig {
    /// Starting day number (default: 1)
    pub initial_day: u32,
}

impl Default for TimeConfig {
    fn default() -> Self {
        Self { initial_day: 1 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // GameDate tests
    #[test]
    fn test_game_date_new() {
        let date = GameDate::new();
        assert_eq!(date.day, 1);
        assert_eq!(date.tick, 0);
    }

    #[test]
    fn test_game_date_increment_day() {
        let mut date = GameDate::new();

        let new_day = date.increment_day();
        assert_eq!(new_day, 2);
        assert_eq!(date.day, 2);

        date.increment_day();
        assert_eq!(date.day, 3);
    }

    #[test]
    fn test_game_date_tick() {
        let mut date = GameDate::new();

        date.tick();
        assert_eq!(date.tick, 1);

        date.tick();
        date.tick();
        assert_eq!(date.tick, 3);
    }

    #[test]
    fn test_game_date_current_day() {
        let date = GameDate::new();
        assert_eq!(date.current_day(), 1);

        let mut date = GameDate::new();
        date.increment_day();
        assert_eq!(date.current_day(), 2);
    }

    #[test]
    fn test_game_date_default() {
        let date = GameDate::default();
        assert_eq!(date.day, 1);
        assert_eq!(date.tick, 0);
    }

    #[test]
    fn test_game_date_independent_day_and_tick() {
        let mut date = GameDate::new();

        // Tick doesn't affect day
        date.tick();
        date.tick();
        assert_eq!(date.day, 1);
        assert_eq!(date.tick, 2);

        // Day doesn't reset tick
        date.increment_day();
        assert_eq!(date.day, 2);
        assert_eq!(date.tick, 2);
    }

    // NextTurnPhase tests
    #[test]
    fn test_next_turn_phase_default() {
        let next_phase = NextTurnPhase::default();
        assert!(!next_phase.is_reserved());
        assert_eq!(next_phase.get(), None);
    }

    #[test]
    fn test_next_turn_phase_reserve() {
        let mut next_phase = NextTurnPhase::default();

        next_phase.reserve(TurnPhase::EnemyTurn);
        assert!(next_phase.is_reserved());
        assert_eq!(next_phase.get(), Some(TurnPhase::EnemyTurn));
    }

    #[test]
    fn test_next_turn_phase_clear() {
        let mut next_phase = NextTurnPhase::default();

        next_phase.reserve(TurnPhase::Processing);
        assert!(next_phase.is_reserved());

        next_phase.clear();
        assert!(!next_phase.is_reserved());
        assert_eq!(next_phase.get(), None);
    }

    #[test]
    fn test_next_turn_phase_overwrite() {
        let mut next_phase = NextTurnPhase::default();

        next_phase.reserve(TurnPhase::Processing);
        assert_eq!(next_phase.get(), Some(TurnPhase::Processing));

        next_phase.reserve(TurnPhase::EnemyTurn);
        assert_eq!(next_phase.get(), Some(TurnPhase::EnemyTurn));
    }

    // TimeConfig tests
    #[test]
    fn test_time_config_default() {
        let config = TimeConfig::default();
        assert_eq!(config.initial_day, 1);
    }

    #[test]
    fn test_time_config_custom() {
        let config = TimeConfig { initial_day: 10 };
        assert_eq!(config.initial_day, 10);
    }

    #[test]
    fn test_time_config_clone() {
        let config1 = TimeConfig { initial_day: 5 };
        let config2 = config1.clone();
        assert_eq!(config1.initial_day, config2.initial_day);
    }
}
