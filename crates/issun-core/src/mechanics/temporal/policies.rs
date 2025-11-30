//! Policy traits for temporal mechanic.
//!
//! This module defines the policy traits that control temporal behavior.

use super::types::{
    ActionCost, ActionType, ActorContext, CalendarConfig, GameDateTime, ResetTrigger, Season,
    TemporalConfig, TemporalPoint,
};

/// Policy for temporal behavior.
///
/// This trait defines how time, action costs, and resets are handled.
/// Different implementations can model different game systems.
pub trait TemporalPolicy {
    /// Get the calendar configuration.
    fn calendar_config(config: &TemporalConfig) -> &CalendarConfig {
        &config.calendar
    }

    /// Calculate base cost for an action type.
    ///
    /// # Arguments
    ///
    /// * `config` - Temporal configuration
    /// * `action` - Type of action
    ///
    /// # Returns
    ///
    /// Base cost before modifiers
    fn base_cost(config: &TemporalConfig, action: &ActionType) -> u32 {
        config.get_base_cost(action)
    }

    /// Calculate cost modifier based on context.
    ///
    /// # Arguments
    ///
    /// * `config` - Temporal configuration
    /// * `action` - Type of action
    /// * `context` - Actor context (skill, buffs, etc.)
    ///
    /// # Returns
    ///
    /// Multiplier for the base cost (1.0 = no change)
    fn cost_modifier(config: &TemporalConfig, action: &ActionType, context: &ActorContext) -> f32;

    /// Calculate final action cost.
    ///
    /// Default implementation uses base_cost * cost_modifier.
    fn calculate_cost(
        config: &TemporalConfig,
        action: &ActionType,
        context: &ActorContext,
    ) -> ActionCost {
        let base = Self::base_cost(config, action);
        let modifier = Self::cost_modifier(config, action, context);
        ActionCost::new(base).apply_modifier(modifier)
    }

    /// Check if budget should reset.
    ///
    /// # Arguments
    ///
    /// * `config` - Temporal configuration
    /// * `old_time` - Previous time
    /// * `new_time` - Current time
    ///
    /// # Returns
    ///
    /// `true` if budget should reset
    fn should_reset(
        config: &TemporalConfig,
        old_time: &TemporalPoint,
        new_time: &TemporalPoint,
    ) -> bool;

    /// Get season for a datetime.
    fn get_season(config: &TemporalConfig, datetime: &GameDateTime) -> Season {
        config.calendar.season(datetime.month)
    }

    /// Calculate time modifier based on time of day.
    ///
    /// Some actions may cost more/less at different times.
    ///
    /// # Returns
    ///
    /// Multiplier for costs (1.0 = no change)
    fn time_of_day_modifier(_config: &TemporalConfig, _datetime: &GameDateTime) -> f32 {
        1.0 // Default: no modifier
    }

    /// Calculate season modifier.
    ///
    /// Some actions may cost more/less in different seasons.
    ///
    /// # Returns
    ///
    /// Multiplier for costs (1.0 = no change)
    fn season_modifier(_config: &TemporalConfig, _season: Season) -> f32 {
        1.0 // Default: no modifier
    }
}

/// Policy for calendar-specific behavior.
///
/// Implement this for custom calendar systems.
pub trait CalendarPolicy {
    /// Get days per month for a specific month/year.
    fn days_per_month(month: u8, year: u32) -> u8;

    /// Get months per year.
    fn months_per_year() -> u8;

    /// Check if a year is a leap year.
    fn is_leap_year(year: u32) -> bool;

    /// Get season for a month.
    fn season(month: u8) -> Season;

    /// Get day of week (0-6, where 0 is the first day of the week).
    fn day_of_week(datetime: &GameDateTime) -> u8 {
        // Default: calculate based on total days
        // This is a simple implementation; override for accuracy
        let days_from_epoch = Self::total_days_from_epoch(datetime);
        (days_from_epoch % 7) as u8
    }

    /// Calculate total days from epoch (year 0, month 1, day 1).
    fn total_days_from_epoch(datetime: &GameDateTime) -> u64 {
        let days_before_year: u64 = (0..datetime.year)
            .map(|y| {
                (1..=Self::months_per_year())
                    .map(|m| Self::days_per_month(m, y) as u64)
                    .sum::<u64>()
            })
            .sum();

        let days_before_month: u64 = (1..datetime.month)
            .map(|m| Self::days_per_month(m, datetime.year) as u64)
            .sum();

        days_before_year + days_before_month + (datetime.day as u64 - 1)
    }
}

/// Policy for action cost calculation.
///
/// Implement this for custom cost systems.
pub trait CostPolicy {
    /// Get base cost for an action.
    fn base_cost(action: &ActionType) -> u32;

    /// Calculate efficiency modifier based on actor context.
    fn efficiency_modifier(context: &ActorContext) -> f32 {
        // Default: skill reduces cost, efficiency bonus applies directly
        let skill_reduction = (context.skill_level * 0.1).min(0.3); // Max 30% reduction
        let efficiency = context.efficiency_bonus;
        1.0 - skill_reduction - efficiency
    }

    /// Apply temporary modifiers (buffs/debuffs).
    fn apply_temporary_modifiers(base_modifier: f32, context: &ActorContext) -> f32 {
        let mut modifier = base_modifier;
        for (_, mod_value) in &context.temporary_modifiers {
            modifier *= 1.0 + mod_value;
        }
        modifier.max(0.1) // Minimum 10% of original cost
    }
}

/// Check if reset should occur based on trigger and time change.
///
/// This is a standalone helper function for standard reset behavior.
pub fn should_reset_by_trigger(
    trigger: &ResetTrigger,
    old_time: &TemporalPoint,
    new_time: &TemporalPoint,
    config: &CalendarConfig,
) -> bool {
    match trigger {
        ResetTrigger::PerTick(interval) => {
            let old_tick = old_time.to_tick(config);
            let new_tick = new_time.to_tick(config);
            (old_tick / interval) != (new_tick / interval)
        }
        ResetTrigger::PerDay => {
            let old_dt = old_time.to_datetime(config);
            let new_dt = new_time.to_datetime(config);
            !old_dt.is_same_day(&new_dt)
        }
        ResetTrigger::PerHour => {
            let old_dt = old_time.to_datetime(config);
            let new_dt = new_time.to_datetime(config);
            old_dt.hour != new_dt.hour || !old_dt.is_same_day(&new_dt)
        }
        ResetTrigger::Continuous => false, // Never reset, just regenerate
        ResetTrigger::OnEvent(_) => false, // Handled externally
    }
}
