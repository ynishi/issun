//! Standard temporal policy implementation.
//!
//! Provides default behavior suitable for most games.

use crate::mechanics::temporal::policies::{should_reset_by_trigger, TemporalPolicy};
use crate::mechanics::temporal::types::{
    ActionType, ActorContext, GameDateTime, Season, TemporalConfig, TemporalPoint,
};

/// Standard temporal policy.
///
/// This policy provides sensible defaults:
/// - Skill level reduces action costs (up to 30%)
/// - Efficiency bonus applies directly
/// - Time of day and season have no effect by default
/// - Reset occurs based on configured trigger
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::temporal::prelude::*;
///
/// // Use default policy
/// type GameTemporal = TemporalMechanic<StandardTemporalPolicy>;
/// ```
pub struct StandardTemporalPolicy;

impl TemporalPolicy for StandardTemporalPolicy {
    fn cost_modifier(
        _config: &TemporalConfig,
        _action: &ActionType,
        context: &ActorContext,
    ) -> f32 {
        // Skill reduces cost (up to 30%)
        let skill_reduction = (context.skill_level * 0.1).min(0.3);

        // Efficiency bonus reduces cost directly
        let efficiency_reduction = context.efficiency_bonus;

        // Apply temporary modifiers
        let mut modifier = 1.0 - skill_reduction - efficiency_reduction;
        for (_, mod_value) in &context.temporary_modifiers {
            modifier *= 1.0 + mod_value;
        }

        modifier.max(0.1) // Minimum 10% of original cost
    }

    fn should_reset(
        config: &TemporalConfig,
        old_time: &TemporalPoint,
        new_time: &TemporalPoint,
    ) -> bool {
        should_reset_by_trigger(&config.reset_trigger, old_time, new_time, &config.calendar)
    }
}

/// Turn-based policy with day-based reset.
///
/// Optimized for traditional turn-based games where:
/// - Actions are discrete
/// - Budget resets at day boundaries
/// - No time-of-day effects
pub struct TurnBasedPolicy;

impl TemporalPolicy for TurnBasedPolicy {
    fn cost_modifier(
        _config: &TemporalConfig,
        _action: &ActionType,
        context: &ActorContext,
    ) -> f32 {
        // Simple skill-based reduction only
        let skill_reduction = (context.skill_level * 0.15).min(0.5);
        (1.0 - skill_reduction).max(0.5) // Minimum 50% of original cost
    }

    fn should_reset(
        config: &TemporalConfig,
        old_time: &TemporalPoint,
        new_time: &TemporalPoint,
    ) -> bool {
        // Always check day boundary for turn-based
        let old_dt = old_time.to_datetime(&config.calendar);
        let new_dt = new_time.to_datetime(&config.calendar);
        !old_dt.is_same_day(&new_dt)
    }
}

/// Real-time policy with continuous regeneration.
///
/// Optimized for action RPGs and real-time games where:
/// - Energy regenerates continuously
/// - No discrete resets
/// - Time of day may affect costs
pub struct RealTimePolicy;

impl TemporalPolicy for RealTimePolicy {
    fn cost_modifier(
        config: &TemporalConfig,
        action: &ActionType,
        context: &ActorContext,
    ) -> f32 {
        let base_modifier = StandardTemporalPolicy::cost_modifier(config, action, context);

        // Real-time games might want time-of-day effects
        // Override time_of_day_modifier for specific games
        base_modifier
    }

    fn should_reset(
        _config: &TemporalConfig,
        _old_time: &TemporalPoint,
        _new_time: &TemporalPoint,
    ) -> bool {
        // Real-time uses continuous regeneration, not discrete resets
        false
    }

    fn time_of_day_modifier(_config: &TemporalConfig, datetime: &GameDateTime) -> f32 {
        // Example: actions cost more at night
        match datetime.time_of_day() {
            crate::mechanics::temporal::types::TimeOfDay::Night => 1.2, // 20% more at night
            _ => 1.0,
        }
    }
}

/// Persona-style policy.
///
/// Combines date tracking with limited daily actions:
/// - Fixed actions per day
/// - Weather/moon phase could affect outcomes
/// - Social links and calendar events matter
pub struct PersonaStylePolicy;

impl TemporalPolicy for PersonaStylePolicy {
    fn cost_modifier(
        _config: &TemporalConfig,
        _action: &ActionType,
        context: &ActorContext,
    ) -> f32 {
        // Most actions cost 1 AP in Persona style
        // Skill might unlock "free" actions
        if context.skill_level >= 1.0 {
            0.0 // Free action for masters
        } else {
            1.0
        }
    }

    fn should_reset(
        config: &TemporalConfig,
        old_time: &TemporalPoint,
        new_time: &TemporalPoint,
    ) -> bool {
        // Reset on day change
        let old_dt = old_time.to_datetime(&config.calendar);
        let new_dt = new_time.to_datetime(&config.calendar);
        !old_dt.is_same_day(&new_dt)
    }

    fn time_of_day_modifier(_config: &TemporalConfig, _datetime: &GameDateTime) -> f32 {
        // In Persona, time slots matter more than continuous time
        // Morning/Afternoon/Evening/Night each have different activities
        1.0 // Activities are either available or not
    }

    fn season_modifier(_config: &TemporalConfig, season: Season) -> f32 {
        // Season might affect certain activities
        match season {
            Season::Summer => 0.9,  // Summer vacation = easier
            Season::Winter => 1.1,  // Winter = harder
            _ => 1.0,
        }
    }
}

/// Strategy game policy.
///
/// For 4X and grand strategy games:
/// - Turn-based with variable costs
/// - Research and building may span multiple turns
/// - Season affects movement and combat
pub struct StrategyGamePolicy;

impl TemporalPolicy for StrategyGamePolicy {
    fn cost_modifier(
        _config: &TemporalConfig,
        action: &ActionType,
        context: &ActorContext,
    ) -> f32 {
        // Different action types have different scaling
        let type_multiplier = match action.0.as_str() {
            "move" => 1.0,
            "attack" => 1.0,
            "build" => 0.8,     // Building benefits more from skill
            "research" => 0.7,  // Research benefits most from skill
            _ => 1.0,
        };

        let skill_bonus = context.skill_level * 0.2;
        (type_multiplier - skill_bonus).max(0.3)
    }

    fn should_reset(
        config: &TemporalConfig,
        old_time: &TemporalPoint,
        new_time: &TemporalPoint,
    ) -> bool {
        // Reset per turn (tick-based)
        let old_tick = old_time.to_tick(&config.calendar);
        let new_tick = new_time.to_tick(&config.calendar);
        old_tick != new_tick
    }

    fn season_modifier(_config: &TemporalConfig, season: Season) -> f32 {
        // Winter makes everything harder
        match season {
            Season::Winter => 1.3, // 30% more expensive in winter
            Season::Spring | Season::Autumn => 1.0,
            Season::Summer => 0.9, // Slightly easier in summer
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mechanics::temporal::types::TemporalPoint;

    #[test]
    fn test_standard_policy_cost_modifier() {
        let config = TemporalConfig::default();
        let action = ActionType::new("test");

        // No skill = no reduction
        let context = ActorContext::default();
        let modifier = StandardTemporalPolicy::cost_modifier(&config, &action, &context);
        assert!((modifier - 1.0).abs() < 0.01);

        // High skill = reduction
        let skilled_context = ActorContext {
            skill_level: 1.0,
            efficiency_bonus: 0.0,
            temporary_modifiers: Default::default(),
        };
        let skilled_modifier =
            StandardTemporalPolicy::cost_modifier(&config, &action, &skilled_context);
        assert!(skilled_modifier < 1.0);
        assert!(skilled_modifier >= 0.7); // Max 30% reduction
    }

    #[test]
    fn test_turn_based_policy_reset() {
        let config = TemporalConfig::turn_based(3);

        let day1 = TemporalPoint::DateTime(GameDateTime::new(1, 1, 1, 10, 0));
        let day1_later = TemporalPoint::DateTime(GameDateTime::new(1, 1, 1, 20, 0));
        let day2 = TemporalPoint::DateTime(GameDateTime::new(1, 1, 2, 10, 0));

        // Same day = no reset
        assert!(!TurnBasedPolicy::should_reset(&config, &day1, &day1_later));

        // Different day = reset
        assert!(TurnBasedPolicy::should_reset(&config, &day1, &day2));
    }

    #[test]
    fn test_realtime_policy_no_reset() {
        let config = TemporalConfig::real_time(100.0, 5.0);

        let time1 = TemporalPoint::Tick(0);
        let time2 = TemporalPoint::Tick(1000);

        // Real-time never resets
        assert!(!RealTimePolicy::should_reset(&config, &time1, &time2));
    }

    #[test]
    fn test_strategy_game_season_modifier() {
        let config = TemporalConfig::default();

        assert!(StrategyGamePolicy::season_modifier(&config, Season::Winter) > 1.0);
        assert!(StrategyGamePolicy::season_modifier(&config, Season::Summer) < 1.0);
    }

    #[test]
    fn test_persona_style_free_action() {
        let config = TemporalConfig::turn_based(3);
        let action = ActionType::new("study");

        // Master gets free action
        let master = ActorContext {
            skill_level: 1.0,
            ..Default::default()
        };
        let modifier = PersonaStylePolicy::cost_modifier(&config, &action, &master);
        assert_eq!(modifier, 0.0);

        // Normal player pays full cost
        let normal = ActorContext::default();
        let modifier = PersonaStylePolicy::cost_modifier(&config, &action, &normal);
        assert_eq!(modifier, 1.0);
    }
}
