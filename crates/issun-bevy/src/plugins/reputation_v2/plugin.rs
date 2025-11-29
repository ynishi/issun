//! ReputationPluginV2: Policy-Based Reputation System for Bevy.
//!
//! This plugin integrates issun-core's policy-based reputation mechanic with Bevy's ECS.

use bevy::prelude::*;
use issun_core::mechanics::reputation::{ReputationConfig, ReputationInput};
use issun_core::mechanics::{ExecutionHint, Mechanic};
use std::marker::PhantomData;

use super::systems::{log_reputation_events, reputation_system};
use super::types::{
    ReputationChangeRequested, ReputationConfigResource, ReputationEventWrapper, ReputationValue,
};

/// SystemSet for sequential reputation execution.
///
/// Used when the mechanic's ExecutionHint indicates non-parallel-safe execution.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct ReputationSequentialSet;

/// Reputation plugin using issun-core's policy-based design.
///
/// This plugin is generic over the reputation mechanic type, allowing you to
/// choose different reputation behaviors at compile time.
///
/// # Type Parameters
///
/// - `M`: The reputation mechanic to use (must implement `Mechanic` with appropriate types)
///
/// # Examples
///
/// ```ignore
/// use bevy::prelude::*;
/// use issun_bevy::plugins::reputation_v2::ReputationPluginV2;
/// use issun_core::mechanics::reputation::prelude::*;
///
/// // Basic reputation (linear change, no decay, hard clamp)
/// type BasicReputation = ReputationMechanic<LinearChange, NoDecay, HardClamp>;
///
/// App::new()
///     .add_plugins(ReputationPluginV2::<BasicReputation>::default())
///     .run();
/// ```
///
/// ```ignore
/// // Skill progression system (logarithmic change for diminishing returns)
/// type SkillSystem = ReputationMechanic<LogarithmicChange, NoDecay, HardClamp>;
///
/// App::new()
///     .add_plugins(ReputationPluginV2::<SkillSystem>::default())
///     .run();
/// ```
///
/// ```ignore
/// // Durability system (linear decay over time)
/// type DurabilitySystem = ReputationMechanic<LinearChange, LinearDecay, ZeroClamp>;
///
/// App::new()
///     .add_plugins(ReputationPluginV2::<DurabilitySystem>::default())
///     .run();
/// ```
pub struct ReputationPluginV2<M>
where
    M: Mechanic<
        Config = ReputationConfig,
        State = issun_core::mechanics::reputation::ReputationState,
        Input = ReputationInput,
        Event = issun_core::mechanics::reputation::ReputationEvent,
    >,
{
    /// Reputation configuration (shared across all entities)
    pub config: ReputationConfig,

    /// Phantom data to hold the mechanic type
    _phantom: PhantomData<M>,
}

impl<M> Default for ReputationPluginV2<M>
where
    M: Mechanic<
        Config = ReputationConfig,
        State = issun_core::mechanics::reputation::ReputationState,
        Input = ReputationInput,
        Event = issun_core::mechanics::reputation::ReputationEvent,
    >,
{
    fn default() -> Self {
        Self {
            config: ReputationConfig::default(),
            _phantom: PhantomData,
        }
    }
}

impl<M> ReputationPluginV2<M>
where
    M: Mechanic<
        Config = ReputationConfig,
        State = issun_core::mechanics::reputation::ReputationState,
        Input = ReputationInput,
        Event = issun_core::mechanics::reputation::ReputationEvent,
    >,
{
    /// Create a new reputation plugin with custom configuration.
    pub fn with_config(config: ReputationConfig) -> Self {
        Self {
            config,
            _phantom: PhantomData,
        }
    }
}

impl<M> Plugin for ReputationPluginV2<M>
where
    M: Mechanic<
            Config = ReputationConfig,
            State = issun_core::mechanics::reputation::ReputationState,
            Input = ReputationInput,
            Event = issun_core::mechanics::reputation::ReputationEvent,
        > + Send
        + Sync
        + 'static,
{
    fn build(&self, app: &mut App) {
        // Register resources - wrap issun-core's config
        app.insert_resource(ReputationConfigResource::new(self.config.clone()));

        // Register component types
        app.register_type::<ReputationConfigResource>();
        app.register_type::<ReputationValue>();

        // Register messages - use wrapper for issun-core events
        app.add_message::<ReputationChangeRequested>();
        app.add_message::<ReputationEventWrapper>();

        // Register systems with execution hints
        // Use ExecutionHint to determine system scheduling
        if M::Execution::PARALLEL_SAFE {
            // Mechanic is parallel-safe - can run concurrently
            app.add_systems(Update, (reputation_system::<M>, log_reputation_events));
            info!(
                "ReputationPluginV2 initialized with mechanic: {} (parallel-safe)",
                std::any::type_name::<M>()
            );
        } else {
            // Mechanic requires sequential execution
            // Check for preferred schedule hint
            if let Some(schedule) = M::Execution::PREFERRED_SCHEDULE {
                info!(
                    "ReputationPluginV2 initialized with mechanic: {} (sequential, schedule: {})",
                    std::any::type_name::<M>(),
                    schedule
                );
                // Note: In real implementation, you'd map string to actual schedule
                // For now, just add to Update with a warning
                warn!(
                    "PREFERRED_SCHEDULE '{}' specified but not yet implemented, using Update",
                    schedule
                );
                app.add_systems(
                    Update,
                    reputation_system::<M>.in_set(ReputationSequentialSet),
                );
            } else {
                // No specific schedule, use sequential set
                info!(
                    "ReputationPluginV2 initialized with mechanic: {} (sequential)",
                    std::any::type_name::<M>()
                );
                app.add_systems(
                    Update,
                    reputation_system::<M>.in_set(ReputationSequentialSet),
                );
            }
            app.add_systems(Update, log_reputation_events);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use issun_core::mechanics::reputation::prelude::*;

    type TestReputation = ReputationMechanic; // Uses defaults

    #[test]
    fn test_plugin_builds() {
        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins);
        app.add_plugins(ReputationPluginV2::<TestReputation>::default());

        // Verify resource exists
        assert!(app.world().contains_resource::<ReputationConfigResource>());
    }

    #[test]
    fn test_plugin_with_custom_config() {
        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins);

        let config = ReputationConfig {
            min: -100.0,
            max: 100.0,
            decay_rate: 0.95,
        };

        app.add_plugins(ReputationPluginV2::<TestReputation>::with_config(
            config.clone(),
        ));

        let resource = app.world().resource::<ReputationConfigResource>();
        assert_eq!(resource.config.min, -100.0);
        assert_eq!(resource.config.max, 100.0);
        assert_eq!(resource.config.decay_rate, 0.95);
    }

    #[test]
    fn test_full_reputation_flow() {
        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins);
        app.add_plugins(ReputationPluginV2::<TestReputation>::default());

        // Spawn entity with reputation
        let entity = app
            .world_mut()
            .spawn((ReputationValue::new(50.0), Name::new("Knight")))
            .id();

        // Request reputation change (+10)
        app.world_mut().write_message(ReputationChangeRequested {
            entity,
            delta: 10.0,
            elapsed_time: 0,
        });

        // Run one update
        app.update();

        // Verify reputation changed
        let reputation = app.world().get::<ReputationValue>(entity).unwrap();
        assert_eq!(reputation.value, 60.0); // 50 + 10
    }

    #[test]
    fn test_reputation_clamping() {
        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins);
        app.add_plugins(ReputationPluginV2::<TestReputation>::default());

        // Spawn entity with reputation near max
        let entity = app
            .world_mut()
            .spawn((ReputationValue::new(95.0), Name::new("Hero")))
            .id();

        // Request large reputation increase
        app.world_mut().write_message(ReputationChangeRequested {
            entity,
            delta: 20.0,
            elapsed_time: 0,
        });

        app.update();

        // Verify clamped to max (100.0)
        let reputation = app.world().get::<ReputationValue>(entity).unwrap();
        assert_eq!(reputation.value, 100.0);
    }

    #[test]
    fn test_skill_progression_plugin() {
        type SkillProgression = ReputationMechanic<LogarithmicChange, NoDecay, HardClamp>;

        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins);
        app.add_plugins(ReputationPluginV2::<SkillProgression>::default());

        // Entity with mid-level skill
        let entity = app
            .world_mut()
            .spawn((ReputationValue::new(50.0), Name::new("Apprentice")))
            .id();

        // Try to gain skill points
        app.world_mut().write_message(ReputationChangeRequested {
            entity,
            delta: 10.0,
            elapsed_time: 0,
        });

        app.update();

        // With logarithmic change, gain should be moderate at mid-level
        let reputation = app.world().get::<ReputationValue>(entity).unwrap();
        assert!((reputation.value - 55.0).abs() < 0.1); // ~55.0 (diminishing returns)
    }

    #[test]
    fn test_durability_system_plugin() {
        type DurabilitySystem = ReputationMechanic<LinearChange, LinearDecay, ZeroClamp>;

        let config = ReputationConfig {
            min: 0.0,
            max: 100.0,
            decay_rate: 0.9, // 10% decay per time unit
        };

        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins);
        app.add_plugins(ReputationPluginV2::<DurabilitySystem>::with_config(config));

        // Tool with full durability
        let entity = app
            .world_mut()
            .spawn((ReputationValue::new(100.0), Name::new("Iron Sword")))
            .id();

        // Use tool (no delta change, but 1 time unit passes)
        app.world_mut().write_message(ReputationChangeRequested {
            entity,
            delta: 0.0,
            elapsed_time: 1,
        });

        app.update();

        // Durability should have decayed by 10
        let reputation = app.world().get::<ReputationValue>(entity).unwrap();
        assert_eq!(reputation.value, 90.0); // 100 - (100 * 0.1)
    }
}
