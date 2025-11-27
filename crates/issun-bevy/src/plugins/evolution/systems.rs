//! Systems for evolution plugin.
//!
//! These systems adapt Bevy's ECS to issun-core's pure evolution logic.

use bevy::{
    ecs::message::{MessageReader, MessageWriter},
    prelude::*,
};
use issun_core::mechanics::evolution::EvolutionInput;
use issun_core::mechanics::Mechanic;

use super::types::{
    BevyEventEmitter, EnvironmentComponent, EvolutionConfigResource, EvolutionEventWrapper,
    EvolutionStateComponent, EvolutionTick,
};

/// System: Handle evolution ticks using the generic Mechanic.
///
/// This system:
/// 1. Reads EvolutionTick messages
/// 2. Queries entity's EvolutionStateComponent and optional EnvironmentComponent
/// 3. Constructs EvolutionInput from ECS data and message
/// 4. Calls Mechanic::step (issun-core)
/// 5. Updates entity's EvolutionStateComponent
/// 6. Emits EvolutionEvent messages
///
/// # Generic Parameters
///
/// - `M`: The evolution mechanic to use (e.g., `FoodDecay`, `OrganicGrowth`)
///
/// # Safety
///
/// This system validates that the entity exists before processing.
/// Invalid entities are skipped with a warning.
pub fn evolution_system<M>(
    mut ticks: MessageReader<EvolutionTick>,
    config: Res<EvolutionConfigResource>,
    mut query: Query<(&mut EvolutionStateComponent, Option<&EnvironmentComponent>)>,
    mut evolution_events: MessageWriter<EvolutionEventWrapper>,
) where
    M: Mechanic<
            Config = issun_core::mechanics::evolution::EvolutionConfig,
            State = issun_core::mechanics::evolution::EvolutionState,
            Input = EvolutionInput,
            Event = issun_core::mechanics::evolution::EvolutionEvent,
        > + Send
        + Sync
        + 'static,
{
    for tick in ticks.read() {
        // Validate entity exists
        let Ok((mut state_component, env_component)) = query.get_mut(tick.entity) else {
            warn!(
                "EvolutionTick: entity {:?} does not exist or has no EvolutionStateComponent",
                tick.entity
            );
            continue;
        };

        // Convert EvolutionStateComponent to issun-core's EvolutionState
        let mut state = state_component.to_evolution_state();

        // Construct EvolutionInput from message and components
        let environment = tick
            .custom_environment
            .as_ref()
            .map(|e| e.to_core_environment())
            .or_else(|| env_component.map(|c| c.to_core_environment()))
            .unwrap_or_default();

        let time_delta = tick.time_delta.unwrap_or(config.config.time_delta);

        let input = EvolutionInput {
            time_delta,
            environment,
        };

        // Create event emitter (wraps Bevy's MessageWriter)
        let mut emitter = BevyEventEmitter::new(tick.entity, &mut evolution_events);

        // Call issun-core's pure evolution logic
        M::step(&config.config, &mut state, input, &mut emitter);

        // Update EvolutionStateComponent from modified state
        state_component.from_evolution_state(&state);
    }
}

/// System: Automatic evolution tick for all active entities.
///
/// This system automatically evolves all entities with EvolutionStateComponent
/// every frame, using the global time delta.
///
/// This is useful for automatic resource regeneration, decay, etc.
/// without requiring manual EvolutionTick messages.
///
/// # Generic Parameters
///
/// - `M`: The evolution mechanic to use
pub fn auto_evolution_system<M>(
    time: Res<Time>,
    config: Res<EvolutionConfigResource>,
    mut query: Query<(
        Entity,
        &mut EvolutionStateComponent,
        Option<&EnvironmentComponent>,
    )>,
    mut evolution_events: MessageWriter<EvolutionEventWrapper>,
) where
    M: Mechanic<
            Config = issun_core::mechanics::evolution::EvolutionConfig,
            State = issun_core::mechanics::evolution::EvolutionState,
            Input = EvolutionInput,
            Event = issun_core::mechanics::evolution::EvolutionEvent,
        > + Send
        + Sync
        + 'static,
{
    let time_delta = time.delta_secs();

    // Skip if no time has passed (first frame or paused)
    if time_delta <= 0.0 {
        return;
    }

    for (entity, mut state_component, env_component) in query.iter_mut() {
        // Convert to issun-core's EvolutionState
        let mut state = state_component.to_evolution_state();

        // Get environment from component or use default
        let environment = env_component
            .map(|c| c.to_core_environment())
            .unwrap_or_default();

        let input = EvolutionInput {
            time_delta,
            environment,
        };

        // Create event emitter
        let mut emitter = BevyEventEmitter::new(entity, &mut evolution_events);

        // Call issun-core's pure evolution logic
        M::step(&config.config, &mut state, input, &mut emitter);

        // Update component from modified state
        state_component.from_evolution_state(&state);
    }
}

/// System: Log evolution events for debugging.
///
/// This system listens to EvolutionEventWrappers and logs them.
/// In a real game, you might use this to trigger VFX, SFX, or UI updates.
pub fn log_evolution_events(mut evolution_events: MessageReader<EvolutionEventWrapper>) {
    use issun_core::mechanics::evolution::EvolutionEvent;

    for wrapper in evolution_events.read() {
        match &wrapper.event {
            EvolutionEvent::ValueChanged {
                old_value,
                new_value,
                delta,
            } => {
                debug!(
                    "Entity {:?}: Value changed from {:.2} to {:.2} (delta: {:.2})",
                    wrapper.entity, old_value, new_value, delta
                );
            }
            EvolutionEvent::MinimumReached { final_value } => {
                info!(
                    "Entity {:?}: Reached minimum (value: {:.2})",
                    wrapper.entity, final_value
                );
            }
            EvolutionEvent::MaximumReached { final_value } => {
                info!(
                    "Entity {:?}: Reached maximum (value: {:.2})",
                    wrapper.entity, final_value
                );
            }
            EvolutionEvent::ThresholdCrossed {
                threshold,
                direction,
            } => {
                info!(
                    "Entity {:?}: Crossed threshold {:.0}% ({:?})",
                    wrapper.entity,
                    threshold * 100.0,
                    direction
                );
            }
            EvolutionEvent::StatusChanged {
                old_status,
                new_status,
            } => {
                info!(
                    "Entity {:?}: Status changed from {:?} to {:?}",
                    wrapper.entity, old_status, new_status
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::evolution::types::SubjectType;
    use issun_core::mechanics::evolution::prelude::*;

    type TestEvolution = SimpleDecay;

    #[test]
    fn test_evolution_system_integration() {
        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins);

        // Register types and messages
        app.insert_resource(EvolutionConfigResource::new(
            issun_core::mechanics::evolution::EvolutionConfig {
                base_rate: 10.0,
                time_delta: 1.0,
            },
        ));
        app.add_message::<EvolutionTick>();
        app.add_message::<EvolutionEventWrapper>();

        // Add system
        app.add_systems(Update, evolution_system::<TestEvolution>);

        // Spawn entity with evolution state
        let entity = app
            .world_mut()
            .spawn(EvolutionStateComponent::new(
                100.0,
                0.0,
                100.0,
                SubjectType::Food,
            ))
            .id();

        // Request evolution
        app.world_mut().write_message(EvolutionTick::new(entity));

        // Run systems
        app.update();

        // Verify value has decayed
        let state = app.world().get::<EvolutionStateComponent>(entity).unwrap();
        assert_eq!(state.value, 90.0); // 100 - 10
    }

    #[test]
    fn test_evolution_system_with_environment() {
        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins);

        app.insert_resource(EvolutionConfigResource::new(
            issun_core::mechanics::evolution::EvolutionConfig {
                base_rate: 10.0,
                time_delta: 1.0,
            },
        ));
        app.add_message::<EvolutionTick>();
        app.add_message::<EvolutionEventWrapper>();

        app.add_systems(Update, evolution_system::<TestEvolution>);

        // Spawn entity with environment component
        let entity = app
            .world_mut()
            .spawn((
                EvolutionStateComponent::new(100.0, 0.0, 100.0, SubjectType::Food),
                EnvironmentComponent::new(30.0, 0.8),
            ))
            .id();

        app.world_mut().write_message(EvolutionTick::new(entity));
        app.update();

        let state = app.world().get::<EvolutionStateComponent>(entity).unwrap();
        assert_eq!(state.value, 90.0);
    }

    #[test]
    fn test_auto_evolution_system() {
        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins);

        app.insert_resource(EvolutionConfigResource::new(
            issun_core::mechanics::evolution::EvolutionConfig {
                base_rate: 5.0,
                time_delta: 1.0,
            },
        ));
        app.add_message::<EvolutionEventWrapper>();

        app.add_systems(Update, auto_evolution_system::<TestEvolution>);

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

        // Value should have decayed automatically
        let state = app.world().get::<EvolutionStateComponent>(entity).unwrap();
        assert!(state.value < 100.0);
    }

    #[test]
    fn test_evolution_to_minimum() {
        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins);

        app.insert_resource(EvolutionConfigResource::new(
            issun_core::mechanics::evolution::EvolutionConfig {
                base_rate: 200.0, // Large rate to reach minimum quickly
                time_delta: 1.0,
            },
        ));
        app.add_message::<EvolutionTick>();
        app.add_message::<EvolutionEventWrapper>();

        app.add_systems(Update, evolution_system::<TestEvolution>);

        let entity = app
            .world_mut()
            .spawn(EvolutionStateComponent::new(
                50.0,
                0.0,
                100.0,
                SubjectType::Food,
            ))
            .id();

        app.world_mut().write_message(EvolutionTick::new(entity));
        app.update();

        let state = app.world().get::<EvolutionStateComponent>(entity).unwrap();
        assert_eq!(state.value, 0.0);
        use crate::plugins::evolution::types::EvolutionStatus;
        assert_eq!(state.status, EvolutionStatus::Depleted);
    }

    #[test]
    fn test_evolution_growth() {
        use issun_core::mechanics::evolution::prelude::SimpleGrowth;

        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins);

        app.insert_resource(EvolutionConfigResource::new(
            issun_core::mechanics::evolution::EvolutionConfig {
                base_rate: 10.0,
                time_delta: 1.0,
            },
        ));
        app.add_message::<EvolutionTick>();
        app.add_message::<EvolutionEventWrapper>();

        app.add_systems(Update, evolution_system::<SimpleGrowth>);

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
}
