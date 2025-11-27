//! CombatPluginV2: Policy-Based Combat System for Bevy.
//!
//! This plugin integrates issun-core's policy-based combat mechanic with Bevy's ECS.

use bevy::prelude::*;
use issun_core::mechanics::combat::{CombatConfig, CombatInput};
use issun_core::mechanics::{ExecutionHint, Mechanic};
use std::marker::PhantomData;

use super::systems::{damage_system, log_combat_events};
use super::types::{
    Attack, CombatConfigResource, CombatEventWrapper, DamageApplied, DamageRequested, Defense,
    ElementType, Health,
};

/// SystemSet for sequential combat execution.
///
/// Used when the mechanic's ExecutionHint indicates non-parallel-safe execution.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct CombatSequentialSet;

/// Combat plugin using issun-core's policy-based design.
///
/// This plugin is generic over the combat mechanic type, allowing you to
/// choose different combat behaviors at compile time.
///
/// # Type Parameters
///
/// - `M`: The combat mechanic to use (must implement `Mechanic` with appropriate types)
///
/// # Examples
///
/// ```ignore
/// use bevy::prelude::*;
/// use issun_bevy::plugins::combat_v2::CombatPluginV2;
/// use issun_core::mechanics::combat::prelude::*;
///
/// // Classic RPG combat
/// type ClassicRPG = CombatMechanic<
///     LinearDamageCalculation,
///     SubtractiveDefense,
///     NoElemental,
/// >;
///
/// App::new()
///     .add_plugins(CombatPluginV2::<ClassicRPG>::default())
///     .run();
/// ```
///
/// ```ignore
/// // Elemental combat (Pokémon-style)
/// type ElementalCombat = CombatMechanic<
///     LinearDamageCalculation,
///     SubtractiveDefense,
///     ElementalAffinity,
/// >;
///
/// App::new()
///     .add_plugins(CombatPluginV2::<ElementalCombat>::default())
///     .run();
/// ```
pub struct CombatPluginV2<M>
where
    M: Mechanic<
        Config = CombatConfig,
        State = issun_core::mechanics::combat::CombatState,
        Input = CombatInput,
        Event = issun_core::mechanics::combat::CombatEvent,
    >,
{
    /// Combat configuration (shared across all entities)
    pub config: CombatConfig,

    /// Phantom data to hold the mechanic type
    _phantom: PhantomData<M>,
}

impl<M> Default for CombatPluginV2<M>
where
    M: Mechanic<
        Config = CombatConfig,
        State = issun_core::mechanics::combat::CombatState,
        Input = CombatInput,
        Event = issun_core::mechanics::combat::CombatEvent,
    >,
{
    fn default() -> Self {
        Self {
            config: CombatConfig::default(),
            _phantom: PhantomData,
        }
    }
}

impl<M> CombatPluginV2<M>
where
    M: Mechanic<
        Config = CombatConfig,
        State = issun_core::mechanics::combat::CombatState,
        Input = CombatInput,
        Event = issun_core::mechanics::combat::CombatEvent,
    >,
{
    /// Create a new combat plugin with custom configuration.
    pub fn with_config(config: CombatConfig) -> Self {
        Self {
            config,
            _phantom: PhantomData,
        }
    }
}

impl<M> Plugin for CombatPluginV2<M>
where
    M: Mechanic<
            Config = CombatConfig,
            State = issun_core::mechanics::combat::CombatState,
            Input = CombatInput,
            Event = issun_core::mechanics::combat::CombatEvent,
        > + Send
        + Sync
        + 'static,
{
    fn build(&self, app: &mut App) {
        // Register resources - wrap issun-core's config
        app.insert_resource(CombatConfigResource::new(self.config.clone()));

        // Register component types
        app.register_type::<CombatConfigResource>();
        app.register_type::<Health>();
        app.register_type::<Attack>();
        app.register_type::<Defense>();
        app.register_type::<ElementType>();

        // Register messages - use wrapper for issun-core events
        app.add_message::<DamageRequested>();
        app.add_message::<DamageApplied>();
        app.add_message::<CombatEventWrapper>();

        // Register systems with execution hints
        // Use ExecutionHint to determine system scheduling
        if M::Execution::PARALLEL_SAFE {
            // Mechanic is parallel-safe - can run concurrently
            app.add_systems(Update, (damage_system::<M>, log_combat_events));
            info!(
                "CombatPluginV2 initialized with mechanic: {} (parallel-safe)",
                std::any::type_name::<M>()
            );
        } else {
            // Mechanic requires sequential execution
            // Check for preferred schedule hint
            if let Some(schedule) = M::Execution::PREFERRED_SCHEDULE {
                info!(
                    "CombatPluginV2 initialized with mechanic: {} (sequential, schedule: {})",
                    std::any::type_name::<M>(),
                    schedule
                );
                // Note: In real implementation, you'd map string to actual schedule
                // For now, just add to Update with a warning
                warn!(
                    "PREFERRED_SCHEDULE '{}' specified but not yet implemented, using Update",
                    schedule
                );
                app.add_systems(Update, damage_system::<M>.in_set(CombatSequentialSet));
            } else {
                // No specific schedule, use sequential set
                info!(
                    "CombatPluginV2 initialized with mechanic: {} (sequential)",
                    std::any::type_name::<M>()
                );
                app.add_systems(Update, damage_system::<M>.in_set(CombatSequentialSet));
            }
            app.add_systems(Update, log_combat_events);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use issun_core::mechanics::combat::prelude::*;

    type TestCombat = CombatMechanic; // Uses defaults

    #[test]
    fn test_plugin_builds() {
        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins);
        app.add_plugins(CombatPluginV2::<TestCombat>::default());

        // Verify resource exists
        assert!(app.world().contains_resource::<CombatConfigResource>());
    }

    #[test]
    fn test_plugin_with_custom_config() {
        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins);

        let config = CombatConfig {
            min_damage: 5,
            critical_multiplier: 3.0,
        };

        app.add_plugins(CombatPluginV2::<TestCombat>::with_config(config.clone()));

        let resource = app.world().resource::<CombatConfigResource>();
        assert_eq!(resource.config.min_damage, 5);
        assert_eq!(resource.config.critical_multiplier, 3.0);
    }

    #[test]
    fn test_full_combat_flow() {
        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins);
        app.add_plugins(CombatPluginV2::<TestCombat>::default());

        // Spawn attacker
        let attacker = app
            .world_mut()
            .spawn((Attack { power: 50 }, Name::new("Knight")))
            .id();

        // Spawn target
        let target = app
            .world_mut()
            .spawn((Health::new(100), Defense { value: 15 }, Name::new("Goblin")))
            .id();

        // Request damage
        app.world_mut()
            .write_message(DamageRequested { attacker, target });

        // Run one update
        app.update();

        // Verify damage was applied
        let health = app.world().get::<Health>(target).unwrap();
        assert_eq!(health.current, 65); // 100 - (50 - 15) = 65
    }

    #[test]
    fn test_elemental_combat_plugin() {
        type ElementalCombat =
            CombatMechanic<LinearDamageCalculation, SubtractiveDefense, ElementalAffinity>;

        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins);
        app.add_plugins(CombatPluginV2::<ElementalCombat>::default());

        use issun_core::mechanics::combat::Element;

        // Fire attacker
        let attacker = app
            .world_mut()
            .spawn((
                Attack { power: 50 },
                ElementType {
                    element: Element::Fire,
                },
                Name::new("Fire Mage"),
            ))
            .id();

        // Ice defender (weak to fire!)
        let target = app
            .world_mut()
            .spawn((
                Health::new(100),
                Defense { value: 10 },
                ElementType {
                    element: Element::Ice,
                },
                Name::new("Ice Golem"),
            ))
            .id();

        app.world_mut()
            .write_message(DamageRequested { attacker, target });

        app.update();

        // Fire vs Ice = 2x multiplier
        // (50 - 10) * 2 = 80 damage
        let health = app.world().get::<Health>(target).unwrap();
        assert_eq!(health.current, 20); // 100 - 80 = 20
    }
}
