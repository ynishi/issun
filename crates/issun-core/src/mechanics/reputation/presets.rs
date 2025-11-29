//! Preset type aliases for common reputation configurations.
//!
//! This module provides ready-to-use reputation mechanic configurations
//! optimized for specific game scenarios. Each preset is a type alias that
//! combines specific strategies to achieve desired behavior.

use super::mechanic::ReputationMechanic;
use super::strategies::*;

/// Basic reputation system with linear changes and hard bounds.
///
/// - Changes: Linear (direct delta application)
/// - Decay: None (permanent reputation)
/// - Clamp: Hard (0-100 range)
///
/// # Use Cases
/// - NPC favorability
/// - Faction standing
/// - Social credit
///
/// # Example
/// ```
/// use issun_core::mechanics::reputation::presets::BasicReputation;
/// use issun_core::mechanics::reputation::{ReputationConfig, ReputationState, ReputationInput};
/// use issun_core::mechanics::{Mechanic, EventEmitter};
///
/// let config = ReputationConfig::default();
/// let mut state = ReputationState::new(50.0);
/// let input = ReputationInput { delta: 10.0, elapsed_time: 0 };
///
/// struct NoOpEmitter;
/// impl EventEmitter<issun_core::mechanics::reputation::ReputationEvent> for NoOpEmitter {
///     fn emit(&mut self, _: issun_core::mechanics::reputation::ReputationEvent) {}
/// }
/// let mut emitter = NoOpEmitter;
///
/// BasicReputation::step(&config, &mut state, input, &mut emitter);
/// assert_eq!(state.value, 60.0);
/// ```
pub type BasicReputation = ReputationMechanic<LinearChange, NoDecay, HardClamp>;

/// Durability system for items/equipment.
///
/// - Changes: Linear (usage decreases durability)
/// - Decay: Linear (natural wear over time)
/// - Clamp: Zero (can't be negative, broken at 0)
///
/// # Use Cases
/// - Weapon/armor durability
/// - Tool lifespan
/// - Building degradation
pub type DurabilitySystem = ReputationMechanic<LinearChange, LinearDecay, ZeroClamp>;

/// Skill progression with diminishing returns.
///
/// - Changes: Logarithmic (harder to level up at high levels)
/// - Decay: Exponential (skills fade without practice)
/// - Clamp: Hard (0-100 range)
///
/// # Use Cases
/// - Skill levels
/// - Proficiency systems
/// - Learning curves
pub type SkillProgression = ReputationMechanic<LogarithmicChange, ExponentialDecay, HardClamp>;

/// Temporary buff/debuff system.
///
/// - Changes: Linear (applied instantly)
/// - Decay: Exponential (fades quickly)
/// - Clamp: Hard (bounded effect)
///
/// # Use Cases
/// - Temporary stat boosts
/// - Status effects
/// - Timed power-ups
pub type TemporaryEffect = ReputationMechanic<LinearChange, ExponentialDecay, HardClamp>;

/// Resource quantity (fuel, ammo, materials).
///
/// - Changes: Linear (consumed/gained directly)
/// - Decay: None (doesn't degrade over time)
/// - Clamp: Zero (can't be negative)
///
/// # Use Cases
/// - Ammunition
/// - Fuel/energy
/// - Crafting materials
/// - Currency
pub type ResourceQuantity = ReputationMechanic<LinearChange, NoDecay, ZeroClamp>;

/// Temperature/environmental metric (unbounded).
///
/// - Changes: Linear (environmental effects)
/// - Decay: Linear (returns to ambient)
/// - Clamp: None (can go arbitrarily high/low)
///
/// # Use Cases
/// - Temperature systems
/// - Pressure/altitude
/// - Pollution levels
pub type EnvironmentalMetric = ReputationMechanic<LinearChange, LinearDecay, NoClamp>;

/// Rank-up system with tier-based progression.
///
/// - Changes: Threshold (easier early, harder late)
/// - Decay: None (ranks are permanent)
/// - Clamp: Hard (defined rank range)
///
/// # Use Cases
/// - Military ranks
/// - Guild levels
/// - Achievement tiers
pub type RankSystem = ReputationMechanic<ThresholdChange, NoDecay, HardClamp>;

/// Mood/morale system with natural return to neutral.
///
/// - Changes: Linear (events affect mood)
/// - Decay: Exponential (returns to neutral over time)
/// - Clamp: Hard (-100 to +100)
///
/// # Use Cases
/// - Character mood
/// - Colonist happiness
/// - Team morale
pub type MoodSystem = ReputationMechanic<LinearChange, ExponentialDecay, HardClamp>;
