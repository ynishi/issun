//! Core types for temporal mechanic.
//!
//! This module provides types for time management, action budgets,
//! and temporal calculations.

use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;

// ============================================================================
// DateTime Types
// ============================================================================

/// Game date and time with minute-level granularity.
///
/// Supports various calendar systems through `CalendarConfig`.
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::temporal::types::GameDateTime;
///
/// let dt = GameDateTime::new(1, 4, 15, 10, 30); // Year 1, April 15, 10:30
/// let later = dt.add_hours(2);
/// assert_eq!(later.hour, 12);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct GameDateTime {
    /// Year (1-based)
    pub year: u32,
    /// Month (1-12 or calendar-specific)
    pub month: u8,
    /// Day of month (1-based)
    pub day: u8,
    /// Hour (0-23)
    pub hour: u8,
    /// Minute (0-59)
    pub minute: u8,
}

impl GameDateTime {
    /// Create a new datetime.
    pub fn new(year: u32, month: u8, day: u8, hour: u8, minute: u8) -> Self {
        Self {
            year,
            month,
            day,
            hour,
            minute,
        }
    }

    /// Create datetime at start of a day.
    pub fn start_of_day(year: u32, month: u8, day: u8) -> Self {
        Self::new(year, month, day, 0, 0)
    }

    /// Create datetime at noon.
    pub fn noon(year: u32, month: u8, day: u8) -> Self {
        Self::new(year, month, day, 12, 0)
    }

    /// Add minutes to this datetime.
    ///
    /// Uses the provided calendar config for overflow handling.
    pub fn add_minutes(&self, minutes: i64, config: &CalendarConfig) -> Self {
        let total_minutes = self.to_total_minutes(config) as i64 + minutes;
        Self::from_total_minutes(total_minutes.max(0) as u64, config)
    }

    /// Add hours to this datetime.
    pub fn add_hours(&self, hours: i64, config: &CalendarConfig) -> Self {
        self.add_minutes(hours * 60, config)
    }

    /// Add days to this datetime.
    pub fn add_days(&self, days: i64, config: &CalendarConfig) -> Self {
        self.add_minutes(days * 24 * 60, config)
    }

    /// Calculate elapsed minutes between two datetimes.
    pub fn elapsed_minutes(&self, other: &Self, config: &CalendarConfig) -> i64 {
        other.to_total_minutes(config) as i64 - self.to_total_minutes(config) as i64
    }

    /// Check if same day as another datetime.
    pub fn is_same_day(&self, other: &Self) -> bool {
        self.year == other.year && self.month == other.month && self.day == other.day
    }

    /// Get time of day classification.
    pub fn time_of_day(&self) -> TimeOfDay {
        match self.hour {
            5..=11 => TimeOfDay::Morning,
            12..=16 => TimeOfDay::Afternoon,
            17..=20 => TimeOfDay::Evening,
            _ => TimeOfDay::Night,
        }
    }

    /// Check if hour is within a range (inclusive).
    pub fn is_within_hours(&self, start_hour: u8, end_hour: u8) -> bool {
        if start_hour <= end_hour {
            self.hour >= start_hour && self.hour <= end_hour
        } else {
            // Wraps around midnight (e.g., 22:00 - 06:00)
            self.hour >= start_hour || self.hour <= end_hour
        }
    }

    /// Convert to tick based on calendar config.
    pub fn to_tick(&self, config: &CalendarConfig) -> u64 {
        self.to_total_minutes(config) * config.ticks_per_minute as u64
    }

    /// Create from tick based on calendar config.
    pub fn from_tick(tick: u64, config: &CalendarConfig) -> Self {
        let total_minutes = tick / config.ticks_per_minute as u64;
        Self::from_total_minutes(total_minutes, config)
    }

    /// Convert to total minutes since epoch.
    fn to_total_minutes(&self, config: &CalendarConfig) -> u64 {
        let days_before_year = (0..self.year)
            .map(|y| config.days_per_year(y))
            .sum::<u32>() as u64;

        let days_before_month: u64 = (1..self.month)
            .map(|m| config.days_per_month(m, self.year) as u64)
            .sum();

        let total_days = days_before_year + days_before_month + (self.day as u64 - 1);
        total_days * 24 * 60 + self.hour as u64 * 60 + self.minute as u64
    }

    /// Create from total minutes since epoch.
    fn from_total_minutes(total_minutes: u64, config: &CalendarConfig) -> Self {
        let minutes_per_day = 24 * 60;

        let mut total_days = total_minutes / minutes_per_day;
        let remaining_minutes = total_minutes % minutes_per_day;

        let hour = (remaining_minutes / 60) as u8;
        let minute = (remaining_minutes % 60) as u8;

        // Find year
        let mut year = 0u32;
        loop {
            let days_in_year = config.days_per_year(year) as u64;
            if total_days < days_in_year {
                break;
            }
            total_days -= days_in_year;
            year += 1;
        }

        // Find month
        let mut month = 1u8;
        loop {
            let days_in_month = config.days_per_month(month, year) as u64;
            if total_days < days_in_month {
                break;
            }
            total_days -= days_in_month;
            month += 1;
            if month > config.months_per_year {
                month = 1;
                break;
            }
        }

        let day = total_days as u8 + 1;

        Self {
            year,
            month,
            day,
            hour,
            minute,
        }
    }
}

impl PartialOrd for GameDateTime {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for GameDateTime {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.year, self.month, self.day, self.hour, self.minute).cmp(&(
            other.year,
            other.month,
            other.day,
            other.hour,
            other.minute,
        ))
    }
}

impl fmt::Display for GameDateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Y{:04}-M{:02}-D{:02} {:02}:{:02}",
            self.year, self.month, self.day, self.hour, self.minute
        )
    }
}

/// Time of day classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimeOfDay {
    /// Early morning to late morning (5:00 - 11:59)
    Morning,
    /// Noon to late afternoon (12:00 - 16:59)
    Afternoon,
    /// Early evening to late evening (17:00 - 20:59)
    Evening,
    /// Night to early morning (21:00 - 4:59)
    Night,
}

/// Season of the year.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Season {
    /// Spring
    Spring,
    /// Summer
    Summer,
    /// Autumn/Fall
    Autumn,
    /// Winter
    Winter,
}

// ============================================================================
// Calendar Configuration
// ============================================================================

/// Calendar system configuration.
///
/// Supports various calendar systems:
/// - Gregorian (standard Earth calendar)
/// - Uniform360 (30 days × 12 months = 360 days)
/// - Custom configurations
#[derive(Debug, Clone)]
pub struct CalendarConfig {
    /// Number of months per year
    pub months_per_year: u8,
    /// Days per month (can vary by month)
    pub days_per_month_table: Vec<u8>,
    /// Whether to use leap years
    pub use_leap_years: bool,
    /// Ticks per minute (for tick conversion)
    pub ticks_per_minute: u32,
    /// Season assignments by month
    pub month_to_season: Vec<Season>,
}

impl CalendarConfig {
    /// Create Gregorian calendar (standard Earth calendar).
    pub fn gregorian() -> Self {
        Self {
            months_per_year: 12,
            days_per_month_table: vec![31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31],
            use_leap_years: true,
            ticks_per_minute: 1,
            month_to_season: vec![
                Season::Winter,  // Jan
                Season::Winter,  // Feb
                Season::Spring,  // Mar
                Season::Spring,  // Apr
                Season::Spring,  // May
                Season::Summer,  // Jun
                Season::Summer,  // Jul
                Season::Summer,  // Aug
                Season::Autumn,  // Sep
                Season::Autumn,  // Oct
                Season::Autumn,  // Nov
                Season::Winter,  // Dec
            ],
        }
    }

    /// Create uniform 360-day calendar (30 days × 12 months).
    ///
    /// Common in simulations and games for simplicity.
    pub fn uniform_360() -> Self {
        Self {
            months_per_year: 12,
            days_per_month_table: vec![30; 12],
            use_leap_years: false,
            ticks_per_minute: 1,
            month_to_season: vec![
                Season::Winter,  // Month 1
                Season::Winter,  // Month 2
                Season::Winter,  // Month 3
                Season::Spring,  // Month 4
                Season::Spring,  // Month 5
                Season::Spring,  // Month 6
                Season::Summer,  // Month 7
                Season::Summer,  // Month 8
                Season::Summer,  // Month 9
                Season::Autumn,  // Month 10
                Season::Autumn,  // Month 11
                Season::Autumn,  // Month 12
            ],
        }
    }

    /// Create a custom calendar.
    pub fn custom(months_per_year: u8, days_per_month: Vec<u8>) -> Self {
        let seasons = (0..months_per_year)
            .map(|m| match m % 4 {
                0 => Season::Winter,
                1 => Season::Spring,
                2 => Season::Summer,
                _ => Season::Autumn,
            })
            .collect();

        Self {
            months_per_year,
            days_per_month_table: days_per_month,
            use_leap_years: false,
            ticks_per_minute: 1,
            month_to_season: seasons,
        }
    }

    /// Get days in a specific month.
    pub fn days_per_month(&self, month: u8, year: u32) -> u8 {
        let idx = (month - 1) as usize;
        let base_days = self.days_per_month_table.get(idx).copied().unwrap_or(30);

        // Handle leap year for February (month 2)
        if self.use_leap_years && month == 2 && Self::is_leap_year(year) {
            base_days + 1
        } else {
            base_days
        }
    }

    /// Get total days in a year.
    pub fn days_per_year(&self, year: u32) -> u32 {
        (1..=self.months_per_year)
            .map(|m| self.days_per_month(m, year) as u32)
            .sum()
    }

    /// Get season for a month.
    pub fn season(&self, month: u8) -> Season {
        let idx = (month - 1) as usize;
        self.month_to_season
            .get(idx)
            .copied()
            .unwrap_or(Season::Spring)
    }

    /// Check if a year is a leap year (Gregorian rules).
    pub fn is_leap_year(year: u32) -> bool {
        (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
    }
}

impl Default for CalendarConfig {
    fn default() -> Self {
        Self::gregorian()
    }
}

// ============================================================================
// Temporal Point (Unified Time Representation)
// ============================================================================

/// Unified temporal point - either tick or datetime.
#[derive(Debug, Clone, PartialEq)]
pub enum TemporalPoint {
    /// Abstract tick counter
    Tick(u64),
    /// Semantic datetime
    DateTime(GameDateTime),
}

impl TemporalPoint {
    /// Convert to tick.
    pub fn to_tick(&self, config: &CalendarConfig) -> u64 {
        match self {
            TemporalPoint::Tick(t) => *t,
            TemporalPoint::DateTime(dt) => dt.to_tick(config),
        }
    }

    /// Convert to datetime.
    pub fn to_datetime(&self, config: &CalendarConfig) -> GameDateTime {
        match self {
            TemporalPoint::Tick(t) => GameDateTime::from_tick(*t, config),
            TemporalPoint::DateTime(dt) => *dt,
        }
    }
}

impl Default for TemporalPoint {
    fn default() -> Self {
        TemporalPoint::Tick(0)
    }
}

// ============================================================================
// Action Budget Types
// ============================================================================

/// Discrete action points (turn-based systems).
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::temporal::types::ActionPoints;
///
/// let mut ap = ActionPoints::new(3);
/// assert!(ap.can_afford(2));
/// ap.consume(2).unwrap();
/// assert_eq!(ap.available, 1);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ActionPoints {
    /// Currently available points
    pub available: u32,
    /// Maximum points (for reset)
    pub max: u32,
}

impl ActionPoints {
    /// Create new action points at max capacity.
    pub fn new(max: u32) -> Self {
        Self {
            available: max,
            max,
        }
    }

    /// Create with specific available/max values.
    pub fn with_available(available: u32, max: u32) -> Self {
        Self {
            available: available.min(max),
            max,
        }
    }

    /// Check if can afford a cost.
    pub fn can_afford(&self, cost: u32) -> bool {
        self.available >= cost
    }

    /// Consume action points.
    pub fn consume(&mut self, cost: u32) -> Result<ConsumeResult, BudgetError> {
        if !self.can_afford(cost) {
            return Err(BudgetError::Insufficient {
                required: cost,
                available: self.available,
            });
        }
        self.available -= cost;
        Ok(ConsumeResult {
            remaining: self.available,
            depleted: self.available == 0,
        })
    }

    /// Reset to max capacity.
    pub fn reset(&mut self) {
        self.available = self.max;
    }

    /// Restore some points (capped at max).
    pub fn restore(&mut self, amount: u32) {
        self.available = (self.available + amount).min(self.max);
    }

    /// Check if depleted.
    pub fn is_depleted(&self) -> bool {
        self.available == 0
    }

    /// Get percentage remaining.
    pub fn percentage(&self) -> f32 {
        if self.max == 0 {
            0.0
        } else {
            self.available as f32 / self.max as f32
        }
    }
}

/// Continuous action energy (real-time systems).
///
/// Energy regenerates over time based on `regen_rate`.
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::temporal::types::ActionEnergy;
///
/// let mut energy = ActionEnergy::new(100.0, 5.0); // 100 max, 5/sec regen
/// energy.consume(50.0).unwrap();
/// energy.regenerate(2.0); // 2 seconds pass
/// assert!((energy.current - 60.0).abs() < 0.01);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ActionEnergy {
    /// Current energy
    pub current: f32,
    /// Maximum energy
    pub max: f32,
    /// Regeneration rate (per second)
    pub regen_rate: f32,
}

impl ActionEnergy {
    /// Create new energy at max capacity.
    pub fn new(max: f32, regen_rate: f32) -> Self {
        Self {
            current: max,
            max,
            regen_rate,
        }
    }

    /// Create with specific current value.
    pub fn with_current(current: f32, max: f32, regen_rate: f32) -> Self {
        Self {
            current: current.min(max),
            max,
            regen_rate,
        }
    }

    /// Check if can afford a cost.
    pub fn can_afford(&self, cost: f32) -> bool {
        self.current >= cost
    }

    /// Consume energy.
    pub fn consume(&mut self, cost: f32) -> Result<ConsumeResult, BudgetError> {
        if !self.can_afford(cost) {
            return Err(BudgetError::InsufficientEnergy {
                required: cost,
                available: self.current,
            });
        }
        self.current -= cost;
        Ok(ConsumeResult {
            remaining: self.current as u32,
            depleted: self.current <= 0.0,
        })
    }

    /// Regenerate energy over time.
    pub fn regenerate(&mut self, delta_seconds: f32) {
        self.current = (self.current + self.regen_rate * delta_seconds).min(self.max);
    }

    /// Calculate time to full energy.
    pub fn time_to_full(&self) -> f32 {
        if self.regen_rate <= 0.0 {
            f32::INFINITY
        } else {
            (self.max - self.current) / self.regen_rate
        }
    }

    /// Reset to max capacity.
    pub fn reset(&mut self) {
        self.current = self.max;
    }

    /// Check if depleted.
    pub fn is_depleted(&self) -> bool {
        self.current <= 0.0
    }

    /// Get percentage remaining.
    pub fn percentage(&self) -> f32 {
        if self.max <= 0.0 {
            0.0
        } else {
            self.current / self.max
        }
    }
}

/// Unified action budget.
#[derive(Debug, Clone)]
pub enum ActionBudget {
    /// Discrete points (turn-based)
    Points(ActionPoints),
    /// Continuous energy (real-time)
    Energy(ActionEnergy),
}

impl ActionBudget {
    /// Check if can afford a cost.
    pub fn can_afford(&self, cost: &ActionCost) -> bool {
        match self {
            ActionBudget::Points(ap) => ap.can_afford(cost.effective),
            ActionBudget::Energy(ae) => ae.can_afford(cost.effective as f32),
        }
    }

    /// Consume from budget.
    pub fn consume(&mut self, cost: &ActionCost) -> Result<ConsumeResult, BudgetError> {
        match self {
            ActionBudget::Points(ap) => ap.consume(cost.effective),
            ActionBudget::Energy(ae) => ae.consume(cost.effective as f32),
        }
    }

    /// Regenerate (for energy) or no-op (for points).
    pub fn regenerate(&mut self, delta_seconds: f32) {
        if let ActionBudget::Energy(ae) = self {
            ae.regenerate(delta_seconds);
        }
    }

    /// Reset budget.
    pub fn reset(&mut self) {
        match self {
            ActionBudget::Points(ap) => ap.reset(),
            ActionBudget::Energy(ae) => ae.reset(),
        }
    }

    /// Check if depleted.
    pub fn is_depleted(&self) -> bool {
        match self {
            ActionBudget::Points(ap) => ap.is_depleted(),
            ActionBudget::Energy(ae) => ae.is_depleted(),
        }
    }
}

impl Default for ActionBudget {
    fn default() -> Self {
        ActionBudget::Points(ActionPoints::new(3))
    }
}

/// Result of consuming from budget.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConsumeResult {
    /// Remaining budget after consumption
    pub remaining: u32,
    /// Whether budget is now depleted
    pub depleted: bool,
}

/// Error when budget operation fails.
#[derive(Debug, Clone, PartialEq)]
pub enum BudgetError {
    /// Not enough action points
    Insufficient {
        /// Required amount
        required: u32,
        /// Actually available
        available: u32,
    },
    /// Not enough energy
    InsufficientEnergy {
        /// Required amount
        required: f32,
        /// Actually available
        available: f32,
    },
}

impl fmt::Display for BudgetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BudgetError::Insufficient {
                required,
                available,
            } => {
                write!(
                    f,
                    "Insufficient action points: need {}, have {}",
                    required, available
                )
            }
            BudgetError::InsufficientEnergy {
                required,
                available,
            } => {
                write!(
                    f,
                    "Insufficient energy: need {:.1}, have {:.1}",
                    required, available
                )
            }
        }
    }
}

impl std::error::Error for BudgetError {}

// ============================================================================
// Action Cost
// ============================================================================

/// Action type identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ActionType(pub String);

impl ActionType {
    /// Create a new action type.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl From<&str> for ActionType {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for ActionType {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Action cost with base and effective values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ActionCost {
    /// Base cost before modifiers
    pub base: u32,
    /// Effective cost after modifiers
    pub effective: u32,
}

impl ActionCost {
    /// Create a new cost with same base and effective.
    pub fn new(base: u32) -> Self {
        Self {
            base,
            effective: base,
        }
    }

    /// Create with explicit effective cost.
    pub fn with_effective(base: u32, effective: u32) -> Self {
        Self { base, effective }
    }

    /// Apply a multiplier to the cost.
    pub fn apply_modifier(&self, modifier: f32) -> Self {
        Self {
            base: self.base,
            effective: (self.base as f32 * modifier).ceil().max(0.0) as u32,
        }
    }

    /// Apply a discount percentage.
    pub fn with_discount(&self, discount_percent: f32) -> Self {
        self.apply_modifier(1.0 - discount_percent / 100.0)
    }

    /// Apply a bonus percentage (increase cost).
    pub fn with_bonus(&self, bonus_percent: f32) -> Self {
        self.apply_modifier(1.0 + bonus_percent / 100.0)
    }

    /// Free action (zero cost).
    pub fn free() -> Self {
        Self::new(0)
    }
}

// ============================================================================
// Time Flow Mode
// ============================================================================

/// Time flow mode.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimeFlowMode {
    /// Turn-based: advance only on explicit request
    Discrete,
    /// Real-time: continuous progression
    Continuous {
        /// Speed multiplier (1.0 = normal)
        speed_multiplier: f32,
    },
    /// Hybrid: real-time with pause capability
    Hybrid {
        /// Whether currently paused
        paused: bool,
        /// Speed when not paused
        speed: f32,
    },
}

impl TimeFlowMode {
    /// Check if time should progress.
    pub fn is_flowing(&self) -> bool {
        match self {
            TimeFlowMode::Discrete => false,
            TimeFlowMode::Continuous { .. } => true,
            TimeFlowMode::Hybrid { paused, .. } => !paused,
        }
    }

    /// Get effective speed multiplier.
    pub fn speed(&self) -> f32 {
        match self {
            TimeFlowMode::Discrete => 0.0,
            TimeFlowMode::Continuous { speed_multiplier } => *speed_multiplier,
            TimeFlowMode::Hybrid { paused, speed } => {
                if *paused {
                    0.0
                } else {
                    *speed
                }
            }
        }
    }
}

impl Default for TimeFlowMode {
    fn default() -> Self {
        TimeFlowMode::Discrete
    }
}

/// When budgets should reset.
#[derive(Debug, Clone, PartialEq)]
pub enum ResetTrigger {
    /// Reset every N ticks
    PerTick(u64),
    /// Reset at day boundary
    PerDay,
    /// Reset at hour boundary
    PerHour,
    /// No automatic reset (continuous regeneration)
    Continuous,
    /// Reset on custom event
    OnEvent(String),
}

impl Default for ResetTrigger {
    fn default() -> Self {
        ResetTrigger::PerDay
    }
}

// ============================================================================
// Global Time Interface
// ============================================================================

/// Interface for accessing global time state.
///
/// Implement this trait to provide global time to temporal mechanic.
/// In ECS systems (like Bevy), this would typically be backed by a Resource.
pub trait GlobalTimeProvider {
    /// Get current tick.
    fn current_tick(&self) -> u64;

    /// Get current datetime.
    fn current_datetime(&self, config: &CalendarConfig) -> GameDateTime {
        GameDateTime::from_tick(self.current_tick(), config)
    }

    /// Get time flow mode.
    fn time_flow_mode(&self) -> TimeFlowMode;

    /// Check if time is paused.
    fn is_paused(&self) -> bool {
        !self.time_flow_mode().is_flowing()
    }
}

/// Interface for controlling global time.
pub trait GlobalTimeController: GlobalTimeProvider {
    /// Advance by ticks.
    fn advance_tick(&mut self, amount: u64);

    /// Advance by minutes.
    fn advance_minutes(&mut self, minutes: u64, config: &CalendarConfig) {
        self.advance_tick(minutes * config.ticks_per_minute as u64);
    }

    /// Set paused state.
    fn set_paused(&mut self, paused: bool);

    /// Set speed multiplier.
    fn set_speed(&mut self, multiplier: f32);
}

// ============================================================================
// Temporal Configuration
// ============================================================================

/// Configuration for temporal mechanic.
#[derive(Debug, Clone)]
pub struct TemporalConfig {
    /// Calendar configuration
    pub calendar: CalendarConfig,
    /// Default action points per entity
    pub default_action_points: u32,
    /// Default energy settings (if using energy)
    pub default_energy_max: f32,
    /// Default energy regen rate
    pub default_energy_regen: f32,
    /// Time flow mode
    pub time_flow_mode: TimeFlowMode,
    /// When budgets reset
    pub reset_trigger: ResetTrigger,
    /// Base costs by action type
    pub action_costs: HashMap<String, u32>,
}

impl Default for TemporalConfig {
    fn default() -> Self {
        Self {
            calendar: CalendarConfig::gregorian(),
            default_action_points: 3,
            default_energy_max: 100.0,
            default_energy_regen: 5.0,
            time_flow_mode: TimeFlowMode::Discrete,
            reset_trigger: ResetTrigger::PerDay,
            action_costs: HashMap::new(),
        }
    }
}

impl TemporalConfig {
    /// Create config for turn-based game.
    pub fn turn_based(actions_per_turn: u32) -> Self {
        Self {
            default_action_points: actions_per_turn,
            time_flow_mode: TimeFlowMode::Discrete,
            reset_trigger: ResetTrigger::PerDay,
            ..Default::default()
        }
    }

    /// Create config for real-time game.
    pub fn real_time(energy_max: f32, regen_rate: f32) -> Self {
        Self {
            default_energy_max: energy_max,
            default_energy_regen: regen_rate,
            time_flow_mode: TimeFlowMode::Continuous {
                speed_multiplier: 1.0,
            },
            reset_trigger: ResetTrigger::Continuous,
            ..Default::default()
        }
    }

    /// Use 360-day calendar.
    pub fn with_360_day_calendar(mut self) -> Self {
        self.calendar = CalendarConfig::uniform_360();
        self
    }

    /// Set action cost.
    pub fn with_action_cost(mut self, action: &str, cost: u32) -> Self {
        self.action_costs.insert(action.to_string(), cost);
        self
    }

    /// Get base cost for an action type.
    pub fn get_base_cost(&self, action: &ActionType) -> u32 {
        self.action_costs.get(&action.0).copied().unwrap_or(1)
    }
}

// ============================================================================
// Temporal State
// ============================================================================

/// Per-entity temporal state.
#[derive(Debug, Clone, Default)]
pub struct TemporalState {
    /// Action budget
    pub budget: ActionBudget,
    /// Last action time
    pub last_action_time: TemporalPoint,
    /// Last reset time
    pub last_reset_time: TemporalPoint,
    /// Actions performed this period
    pub actions_this_period: u32,
}

impl TemporalState {
    /// Create with action points.
    pub fn with_points(max: u32) -> Self {
        Self {
            budget: ActionBudget::Points(ActionPoints::new(max)),
            ..Default::default()
        }
    }

    /// Create with energy.
    pub fn with_energy(max: f32, regen_rate: f32) -> Self {
        Self {
            budget: ActionBudget::Energy(ActionEnergy::new(max, regen_rate)),
            ..Default::default()
        }
    }
}

// ============================================================================
// Temporal Input
// ============================================================================

/// Input for temporal mechanic step.
#[derive(Debug, Clone, Default)]
pub struct TemporalInput {
    /// Current time
    pub current_time: TemporalPoint,
    /// Delta time in seconds (for real-time)
    pub delta_seconds: Option<f32>,
    /// Requested action to perform
    pub requested_action: Option<ActionRequest>,
    /// Actor context for cost calculation
    pub actor_context: ActorContext,
}

/// Request to perform an action.
#[derive(Debug, Clone)]
pub struct ActionRequest {
    /// Type of action
    pub action_type: ActionType,
    /// Override cost (if any)
    pub override_cost: Option<u32>,
}

impl ActionRequest {
    /// Create a new action request.
    pub fn new(action_type: impl Into<ActionType>) -> Self {
        Self {
            action_type: action_type.into(),
            override_cost: None,
        }
    }

    /// Create with explicit cost.
    pub fn with_cost(action_type: impl Into<ActionType>, cost: u32) -> Self {
        Self {
            action_type: action_type.into(),
            override_cost: Some(cost),
        }
    }
}

/// Actor context for cost calculation.
#[derive(Debug, Clone, Default)]
pub struct ActorContext {
    /// Actor's skill level (affects cost modifiers)
    pub skill_level: f32,
    /// Actor's efficiency bonus
    pub efficiency_bonus: f32,
    /// Temporary modifiers (e.g., buffs)
    pub temporary_modifiers: HashMap<String, f32>,
}

// ============================================================================
// Temporal Events
// ============================================================================

/// Events emitted by temporal mechanic.
#[derive(Debug, Clone)]
pub enum TemporalEvent {
    /// Action was consumed successfully
    ActionConsumed {
        /// Action type
        action_type: ActionType,
        /// Cost paid
        cost: ActionCost,
        /// Remaining budget
        remaining: u32,
    },
    /// Action was rejected (insufficient budget)
    ActionRejected {
        /// Action type
        action_type: ActionType,
        /// Error reason
        reason: BudgetError,
    },
    /// Budget was depleted
    BudgetDepleted,
    /// Budget was reset
    BudgetReset {
        /// New budget after reset
        new_available: u32,
    },
    /// Energy regenerated
    EnergyRegenerated {
        /// Amount regenerated
        amount: f32,
        /// New current value
        current: f32,
    },
    /// Time advanced
    TimeAdvanced {
        /// Previous time
        from: TemporalPoint,
        /// New time
        to: TemporalPoint,
    },
    /// Day changed
    DayChanged {
        /// New day
        new_day: u8,
        /// New month
        new_month: u8,
        /// New year
        new_year: u32,
    },
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_datetime_creation() {
        let dt = GameDateTime::new(1, 4, 15, 10, 30);
        assert_eq!(dt.year, 1);
        assert_eq!(dt.month, 4);
        assert_eq!(dt.day, 15);
        assert_eq!(dt.hour, 10);
        assert_eq!(dt.minute, 30);
    }

    #[test]
    fn test_game_datetime_add_minutes() {
        let config = CalendarConfig::gregorian();
        let dt = GameDateTime::new(1, 1, 1, 10, 30);

        let later = dt.add_minutes(45, &config);
        assert_eq!(later.hour, 11);
        assert_eq!(later.minute, 15);
    }

    #[test]
    fn test_game_datetime_add_hours_overflow() {
        let config = CalendarConfig::gregorian();
        let dt = GameDateTime::new(1, 1, 1, 23, 0);

        let later = dt.add_hours(2, &config);
        assert_eq!(later.day, 2);
        assert_eq!(later.hour, 1);
    }

    #[test]
    fn test_game_datetime_time_of_day() {
        assert_eq!(
            GameDateTime::new(1, 1, 1, 6, 0).time_of_day(),
            TimeOfDay::Morning
        );
        assert_eq!(
            GameDateTime::new(1, 1, 1, 14, 0).time_of_day(),
            TimeOfDay::Afternoon
        );
        assert_eq!(
            GameDateTime::new(1, 1, 1, 19, 0).time_of_day(),
            TimeOfDay::Evening
        );
        assert_eq!(
            GameDateTime::new(1, 1, 1, 23, 0).time_of_day(),
            TimeOfDay::Night
        );
    }

    #[test]
    fn test_calendar_gregorian() {
        let config = CalendarConfig::gregorian();
        assert_eq!(config.days_per_month(1, 2000), 31); // January
        assert_eq!(config.days_per_month(2, 2000), 29); // Feb leap year
        assert_eq!(config.days_per_month(2, 2001), 28); // Feb non-leap
        assert_eq!(config.days_per_year(2000), 366);
        assert_eq!(config.days_per_year(2001), 365);
    }

    #[test]
    fn test_calendar_uniform_360() {
        let config = CalendarConfig::uniform_360();
        for month in 1..=12 {
            assert_eq!(config.days_per_month(month, 1), 30);
        }
        assert_eq!(config.days_per_year(1), 360);
    }

    #[test]
    fn test_action_points_consume() {
        let mut ap = ActionPoints::new(5);
        assert!(ap.can_afford(3));

        let result = ap.consume(3).unwrap();
        assert_eq!(result.remaining, 2);
        assert!(!result.depleted);

        let result = ap.consume(2).unwrap();
        assert_eq!(result.remaining, 0);
        assert!(result.depleted);

        assert!(ap.consume(1).is_err());
    }

    #[test]
    fn test_action_points_reset() {
        let mut ap = ActionPoints::new(5);
        ap.consume(3).unwrap();
        assert_eq!(ap.available, 2);

        ap.reset();
        assert_eq!(ap.available, 5);
    }

    #[test]
    fn test_action_energy_regenerate() {
        let mut energy = ActionEnergy::new(100.0, 10.0);
        energy.consume(50.0).unwrap();
        assert!((energy.current - 50.0).abs() < 0.01);

        energy.regenerate(2.0); // 2 seconds * 10/sec = 20
        assert!((energy.current - 70.0).abs() < 0.01);

        energy.regenerate(10.0); // Would be 170, but capped at max
        assert!((energy.current - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_action_cost_modifiers() {
        let cost = ActionCost::new(10);
        assert_eq!(cost.effective, 10);

        let discounted = cost.with_discount(20.0); // 20% off
        assert_eq!(discounted.effective, 8);

        let increased = cost.with_bonus(50.0); // 50% more
        assert_eq!(increased.effective, 15);
    }

    #[test]
    fn test_time_flow_mode() {
        let discrete = TimeFlowMode::Discrete;
        assert!(!discrete.is_flowing());
        assert_eq!(discrete.speed(), 0.0);

        let continuous = TimeFlowMode::Continuous {
            speed_multiplier: 2.0,
        };
        assert!(continuous.is_flowing());
        assert_eq!(continuous.speed(), 2.0);

        let hybrid_paused = TimeFlowMode::Hybrid {
            paused: true,
            speed: 1.0,
        };
        assert!(!hybrid_paused.is_flowing());
        assert_eq!(hybrid_paused.speed(), 0.0);

        let hybrid_running = TimeFlowMode::Hybrid {
            paused: false,
            speed: 1.5,
        };
        assert!(hybrid_running.is_flowing());
        assert_eq!(hybrid_running.speed(), 1.5);
    }

    #[test]
    fn test_datetime_ordering() {
        let dt1 = GameDateTime::new(1, 1, 1, 10, 0);
        let dt2 = GameDateTime::new(1, 1, 1, 11, 0);
        let dt3 = GameDateTime::new(1, 1, 2, 9, 0);

        assert!(dt1 < dt2);
        assert!(dt2 < dt3);
        assert!(dt1 < dt3);
    }

    #[test]
    fn test_tick_datetime_conversion() {
        let config = CalendarConfig::uniform_360();
        let dt = GameDateTime::new(0, 1, 1, 12, 30);

        let tick = dt.to_tick(&config);
        let restored = GameDateTime::from_tick(tick, &config);

        assert_eq!(restored, dt);
    }

    #[test]
    fn test_temporal_config_builders() {
        let turn_based = TemporalConfig::turn_based(5);
        assert_eq!(turn_based.default_action_points, 5);
        assert!(matches!(turn_based.time_flow_mode, TimeFlowMode::Discrete));

        let real_time = TemporalConfig::real_time(200.0, 10.0);
        assert!((real_time.default_energy_max - 200.0).abs() < 0.01);
        assert!(matches!(
            real_time.time_flow_mode,
            TimeFlowMode::Continuous { .. }
        ));
    }
}
