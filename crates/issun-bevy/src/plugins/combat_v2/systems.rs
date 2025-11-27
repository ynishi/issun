//! Systems for combat_v2 plugin.
//!
//! These systems adapt Bevy's ECS to issun-core's pure combat logic.

use bevy::{
    ecs::message::{MessageReader, MessageWriter},
    prelude::*,
};
use issun_core::mechanics::combat::{CombatConfig, CombatInput};
use issun_core::mechanics::Mechanic;

use super::types::{
    Attack, BevyEventEmitter, CombatConfigResource, CombatEventWrapper, DamageRequested, Defense,
    ElementType, Health,
};

/// System: Handle damage requests using the generic Mechanic.
///
/// This system:
/// 1. Reads DamageRequested messages
/// 2. Queries attacker and defender components
/// 3. Constructs CombatInput from ECS data
/// 4. Calls Mechanic::step (issun-core)
/// 5. Updates defender's Health component
/// 6. Emits DamageApplied messages
///
/// # Generic Parameters
///
/// - `M`: The combat mechanic to use (e.g., `CombatMechanic<Linear, Subtractive, NoElemental>`)
///
/// # Safety
///
/// This system validates that both attacker and target entities exist before
/// processing. Invalid entities are skipped with a warning.
pub fn damage_system<M>(
    mut requests: MessageReader<DamageRequested>,
    config: Res<CombatConfigResource>,
    attacker_query: Query<(&Attack, Option<&ElementType>)>,
    mut defender_query: Query<(&mut Health, &Defense, Option<&ElementType>)>,
    mut combat_events: MessageWriter<CombatEventWrapper>,
) where
    M: Mechanic<
            Config = CombatConfig,
            State = issun_core::mechanics::combat::CombatState,
            Input = CombatInput,
            Event = issun_core::mechanics::combat::CombatEvent,
        > + Send
        + Sync
        + 'static,
{
    for request in requests.read() {
        // Validate attacker exists
        let Ok((attacker_attack, attacker_element)) = attacker_query.get(request.attacker) else {
            warn!(
                "DamageRequested: attacker {:?} does not exist or has no Attack component",
                request.attacker
            );
            continue;
        };

        // Validate target exists
        let Ok((mut target_health, target_defense, target_element)) =
            defender_query.get_mut(request.target)
        else {
            warn!(
                "DamageRequested: target {:?} does not exist or has no Health/Defense component",
                request.target
            );
            continue;
        };

        // 1. Convert Health to CombatState
        let mut state = target_health.to_combat_state();

        // 2. Construct CombatInput from ECS components
        let input = CombatInput {
            attacker_power: attacker_attack.power,
            defender_defense: target_defense.value,
            attacker_element: attacker_element.map(|e| e.element),
            defender_element: target_element.map(|e| e.element),
        };

        // 3. Create event emitter (wraps Bevy's MessageWriter)
        let mut emitter = BevyEventEmitter::new(request.target, &mut combat_events);

        // 4. Call issun-core's pure combat logic
        M::step(&config.config, &mut state, input, &mut emitter);

        // 5. Update Health component from modified state
        target_health.from_combat_state(&state);
    }
}

/// System: Log combat events for debugging.
///
/// This system listens to CombatEventWrappers and logs them.
/// In a real game, you might use this to trigger VFX, SFX, or UI updates.
pub fn log_combat_events(mut combat_events: MessageReader<CombatEventWrapper>) {
    for wrapper in combat_events.read() {
        match wrapper.event {
            issun_core::mechanics::combat::CombatEvent::DamageDealt {
                amount,
                is_critical,
                is_fatal,
            } => {
                info!(
                    "Entity {:?} took {} damage (critical: {}, fatal: {})",
                    wrapper.entity, amount, is_critical, is_fatal
                );
            }
            issun_core::mechanics::combat::CombatEvent::Blocked { attempted_damage } => {
                info!(
                    "Entity {:?} blocked the attack (negated {} damage)",
                    wrapper.entity, attempted_damage
                );
            }
            issun_core::mechanics::combat::CombatEvent::Evaded => {
                info!("Entity {:?} evaded the attack", wrapper.entity);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::combat_v2::DamageApplied;
    use issun_core::mechanics::combat::prelude::*;

    type TestCombat = CombatMechanic; // Uses defaults

    #[test]
    fn test_damage_system_integration() {
        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins);

        // Register types and messages
        app.insert_resource(CombatConfigResource::new(CombatConfig::default()));
        app.add_message::<DamageRequested>();
        app.add_message::<DamageApplied>();
        app.add_message::<CombatEventWrapper>();

        // Add system
        app.add_systems(Update, damage_system::<TestCombat>);

        // Spawn attacker
        let attacker = app
            .world_mut()
            .spawn((Attack { power: 30 }, Name::new("Attacker")))
            .id();

        // Spawn target
        let target = app
            .world_mut()
            .spawn((Health::new(100), Defense { value: 10 }, Name::new("Target")))
            .id();

        // Request damage
        app.world_mut()
            .write_message(DamageRequested { attacker, target });

        // Run systems
        app.update();

        // Verify damage was applied
        let health = app.world().get::<Health>(target).unwrap();
        assert_eq!(health.current, 80); // 100 - (30 - 10) = 80
        assert!(health.is_alive());
    }

    #[test]
    fn test_damage_system_fatal() {
        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins);

        app.insert_resource(CombatConfigResource::new(CombatConfig::default()));
        app.add_message::<DamageRequested>();
        app.add_message::<DamageApplied>();
        app.add_message::<CombatEventWrapper>();

        app.add_systems(Update, damage_system::<TestCombat>);

        let attacker = app
            .world_mut()
            .spawn((Attack { power: 200 }, Name::new("Attacker")))
            .id();

        let target = app
            .world_mut()
            .spawn((Health::new(50), Defense { value: 0 }, Name::new("Target")))
            .id();

        app.world_mut()
            .write_message(DamageRequested { attacker, target });

        app.update();

        let health = app.world().get::<Health>(target).unwrap();
        assert_eq!(health.current, 0);
        assert!(health.is_dead());
    }
}
