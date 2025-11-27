//! EvolutionPlugin: Policy-Based Evolution System for Bevy.
//!
//! This plugin integrates issun-core's policy-based evolution mechanic with Bevy's ECS.

use bevy::prelude::*;
use issun_core::mechanics::evolution::EvolutionInput;
use issun_core::mechanics::{ExecutionHint, Mechanic};
use std::marker::PhantomData;

use super::systems::{auto_evolution_system, evolution_system, log_evolution_events};
#[cfg(test)]
use super::types::EnvironmentComponent;
use super::types::{
    EvolutionConfigResource, EvolutionEventWrapper, EvolutionStateComponent, EvolutionTick,
    SubjectType,
};

/// SystemSet for sequential evolution execution.
///
/// Used when the mechanic's ExecutionHint indicates non-parallel-safe execution.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct EvolutionSequentialSet;

/// Evolution plugin using issun-core's policy-based design.
///
/// This plugin is generic over the evolution mechanic type, allowing you to
/// choose different evolution behaviors at compile time.
///
/// # Type Parameters
///
/// - `M`: The evolution mechanic to use (must implement `Mechanic` with appropriate types)
///
/// # Configuration
///
/// - `config`: Evolution configuration (base rate, time delta)
/// - `auto_tick`: If true, automatically evolves all entities every frame
/// - `log_events`: If true, logs evolution events for debugging
///
/// # Examples
///
/// ```ignore
/// use bevy::prelude::*;
/// use issun_bevy::plugins::evolution::EvolutionPlugin;
/// use issun_core::mechanics::evolution::prelude::*;
///
/// // Food spoilage system
/// type FoodSpoilage = EvolutionMechanic<Decay, HumidityBased, ExponentialRate>;
///
/// App::new()
///     .add_plugins(EvolutionPlugin::<FoodSpoilage>::default())
///     .run();
/// ```
///
/// ```ignore
/// // Plant growth with auto-tick disabled (manual control)
/// type PlantGrowth = EvolutionMechanic<Growth, TemperatureBased, LinearRate>;
///
/// App::new()
///     .add_plugins(
///         EvolutionPlugin::<PlantGrowth>::new(
///             EvolutionConfig { base_rate: 2.0, time_delta: 1.0 }
///         )
///         .with_auto_tick(false)
///     )
///     .run();
/// ```
pub struct EvolutionPlugin<M>
where
    M: Mechanic<
        Config = issun_core::mechanics::evolution::EvolutionConfig,
        State = issun_core::mechanics::evolution::EvolutionState,
        Input = EvolutionInput,
        Event = issun_core::mechanics::evolution::EvolutionEvent,
    >,
{
    /// Evolution configuration (shared across all entities)
    pub config: issun_core::mechanics::evolution::EvolutionConfig,

    /// Enable automatic evolution every frame
    pub auto_tick: bool,

    /// Enable event logging
    pub log_events: bool,

    /// Phantom data to hold the mechanic type
    _phantom: PhantomData<M>,
}

impl<M> Default for EvolutionPlugin<M>
where
    M: Mechanic<
        Config = issun_core::mechanics::evolution::EvolutionConfig,
        State = issun_core::mechanics::evolution::EvolutionState,
        Input = EvolutionInput,
        Event = issun_core::mechanics::evolution::EvolutionEvent,
    >,
{
    fn default() -> Self {
        Self {
            config: issun_core::mechanics::evolution::EvolutionConfig::default(),
            auto_tick: true,  // Enable auto-tick by default
            log_events: true, // Enable logging by default
            _phantom: PhantomData,
        }
    }
}

impl<M> EvolutionPlugin<M>
where
    M: Mechanic<
        Config = issun_core::mechanics::evolution::EvolutionConfig,
        State = issun_core::mechanics::evolution::EvolutionState,
        Input = EvolutionInput,
        Event = issun_core::mechanics::evolution::EvolutionEvent,
    >,
{
    /// Create a new evolution plugin with custom configuration.
    pub fn new(config: issun_core::mechanics::evolution::EvolutionConfig) -> Self {
        Self {
            config,
            auto_tick: true,
            log_events: true,
            _phantom: PhantomData,
        }
    }

    /// Set whether to enable automatic evolution every frame.
    pub fn with_auto_tick(mut self, auto_tick: bool) -> Self {
        self.auto_tick = auto_tick;
        self
    }

    /// Set whether to enable event logging.
    pub fn with_log_events(mut self, log_events: bool) -> Self {
        self.log_events = log_events;
        self
    }
}

impl<M> Plugin for EvolutionPlugin<M>
where
    M: Mechanic<
            Config = issun_core::mechanics::evolution::EvolutionConfig,
            State = issun_core::mechanics::evolution::EvolutionState,
            Input = EvolutionInput,
            Event = issun_core::mechanics::evolution::EvolutionEvent,
        > + Send
        + Sync
        + 'static,
{
    fn build(&self, app: &mut App) {
        // Register resources - wrap issun-core's config
        app.insert_resource(EvolutionConfigResource::new(self.config.clone()));

        // Register component types
        app.register_type::<EvolutionConfigResource>();
        app.register_type::<EvolutionStateComponent>();
        app.register_type::<SubjectType>();

        // Register messages
        app.add_message::<EvolutionTick>();
        app.add_message::<EvolutionEventWrapper>();

        // Register systems with execution hints
        if M::Execution::PARALLEL_SAFE {
            // Mechanic is parallel-safe - can run concurrently
            if self.auto_tick && self.log_events {
                app.add_systems(
                    Update,
                    (
                        evolution_system::<M>,
                        auto_evolution_system::<M>,
                        log_evolution_events,
                    ),
                );
            } else if self.auto_tick {
                app.add_systems(Update, (evolution_system::<M>, auto_evolution_system::<M>));
            } else if self.log_events {
                app.add_systems(Update, (evolution_system::<M>, log_evolution_events));
            } else {
                app.add_systems(Update, evolution_system::<M>);
            }

            info!(
                "EvolutionPlugin initialized with mechanic: {} (parallel-safe, auto_tick: {}, log_events: {})",
                std::any::type_name::<M>(),
                self.auto_tick,
                self.log_events
            );
        } else {
            // Mechanic requires sequential execution
            app.add_systems(Update, evolution_system::<M>.in_set(EvolutionSequentialSet));

            if self.auto_tick {
                app.add_systems(
                    Update,
                    auto_evolution_system::<M>.in_set(EvolutionSequentialSet),
                );
            }

            if self.log_events {
                app.add_systems(Update, log_evolution_events);
            }

            info!(
                "EvolutionPlugin initialized with mechanic: {} (sequential, auto_tick: {}, log_events: {})",
                std::any::type_name::<M>(),
                self.auto_tick,
                self.log_events
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use issun_core::mechanics::evolution::prelude::{FoodDecay, SimpleDecay, SimpleGrowth};

    type TestEvolution = SimpleDecay;

    #[test]
    fn test_plugin_builds() {
        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins);
        app.add_plugins(EvolutionPlugin::<TestEvolution>::default());

        // Verify resource exists
        assert!(app.world().contains_resource::<EvolutionConfigResource>());
    }

    #[test]
    fn test_plugin_with_custom_config() {
        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins);

        let config = issun_core::mechanics::evolution::EvolutionConfig {
            base_rate: 5.0,
            time_delta: 0.5,
        };

        app.add_plugins(EvolutionPlugin::<TestEvolution>::new(config.clone()));

        let resource = app.world().resource::<EvolutionConfigResource>();
        assert_eq!(resource.config.base_rate, 5.0);
        assert_eq!(resource.config.time_delta, 0.5);
    }

    #[test]
    fn test_full_evolution_flow_manual() {
        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins);
        app.add_plugins(
            EvolutionPlugin::<TestEvolution>::default()
                .with_auto_tick(false) // Manual control
                .with_log_events(false),
        );

        // Spawn entity
        let entity = app
            .world_mut()
            .spawn(EvolutionStateComponent::new(
                100.0,
                0.0,
                100.0,
                SubjectType::Food,
            ))
            .id();

        // Manually request evolution
        app.world_mut().write_message(EvolutionTick::new(entity));

        // Run one update
        app.update();

        // Verify value decayed
        let state = app.world().get::<EvolutionStateComponent>(entity).unwrap();
        assert_eq!(state.value, 99.0); // 100 - 1 (default base_rate)
    }

    #[test]
    fn test_full_evolution_flow_auto() {
        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins);
        app.add_plugins(
            EvolutionPlugin::<TestEvolution>::new(
                issun_core::mechanics::evolution::EvolutionConfig {
                    base_rate: 5.0,
                    time_delta: 1.0,
                },
            )
            .with_auto_tick(true)
            .with_log_events(false),
        );

        // Spawn entity
        let entity = app
            .world_mut()
            .spawn(EvolutionStateComponent::new(
                100.0,
                0.0,
                100.0,
                SubjectType::Resource,
            ))
            .id();

        // Run update twice (first update has delta_secs = 0)
        app.update();
        app.update();

        // Verify value decayed automatically
        let state = app.world().get::<EvolutionStateComponent>(entity).unwrap();
        assert!(state.value < 100.0);
    }

    #[test]
    fn test_growth_mechanic() {
        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins);
        app.add_plugins(
            EvolutionPlugin::<SimpleGrowth>::new(
                issun_core::mechanics::evolution::EvolutionConfig {
                    base_rate: 10.0,
                    time_delta: 1.0,
                },
            )
            .with_auto_tick(false)
            .with_log_events(false),
        );

        let entity = app
            .world_mut()
            .spawn(EvolutionStateComponent::new(
                10.0,
                0.0,
                100.0,
                SubjectType::Plant,
            ))
            .id();

        app.world_mut().write_message(EvolutionTick::new(entity));
        app.update();

        let state = app.world().get::<EvolutionStateComponent>(entity).unwrap();
        assert_eq!(state.value, 20.0); // 10 + 10
    }

    #[test]
    fn test_food_decay_preset() {
        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins);
        app.add_plugins(
            EvolutionPlugin::<FoodDecay>::new(issun_core::mechanics::evolution::EvolutionConfig {
                base_rate: 5.0,
                time_delta: 1.0,
            })
            .with_auto_tick(false)
            .with_log_events(false),
        );

        // High humidity environment
        let entity = app
            .world_mut()
            .spawn((
                EvolutionStateComponent::new(100.0, 0.0, 100.0, SubjectType::Food),
                EnvironmentComponent::new(25.0, 0.9), // High humidity
            ))
            .id();

        app.world_mut().write_message(EvolutionTick::new(entity));
        app.update();

        let state = app.world().get::<EvolutionStateComponent>(entity).unwrap();
        // Should decay faster due to high humidity (HumidityBased policy)
        // and exponential rate
        assert!(state.value < 100.0);
    }
}
