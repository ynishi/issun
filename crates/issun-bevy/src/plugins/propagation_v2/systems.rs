//! Systems for propagation_v2 plugin.
//!
//! These systems adapt Bevy's ECS to issun-core's pure propagation logic.

use bevy::{ecs::message::MessageWriter, prelude::*};
use issun_core::mechanics::propagation::{PropagationEvent, PropagationGraph, PropagationInput};
use issun_core::mechanics::Mechanic;
use std::collections::HashMap;

use super::types::{
    BevyPropagationEmitter, NodeSeverity, PropagationEventWrapper, PropagationGraphResource,
    PropagationStateComponent,
};

/// System: Run propagation step using the generic Mechanic.
///
/// This system:
/// 1. Collects node severities from ECS components
/// 2. Constructs PropagationInput from node states
/// 3. Calls Mechanic::step (issun-core)
/// 4. Emits PropagationEventWrappers
///
/// # Generic Parameters
///
/// - `M`: The propagation mechanic to use (e.g., `LinearPropagationMechanic`)
///
/// # Note
///
/// This system does NOT automatically update NodeSeverity components based on
/// InitialInfection events. That's a separate concern handled by game logic.
/// The propagation mechanic only calculates pressures and emits events.
pub fn propagation_step_system<M>(
    graph: Res<PropagationGraphResource>,
    mut state_query: Query<&mut PropagationStateComponent<M>>,
    node_query: Query<&NodeSeverity>,
    mut event_writer: MessageWriter<PropagationEventWrapper>,
) where
    M: Mechanic<
            Config = PropagationGraph,
            State = issun_core::mechanics::propagation::PropagationState,
            Input = PropagationInput,
            Event = PropagationEvent,
        > + Send
        + Sync
        + 'static,
{
    // Get or skip if no state component exists
    let Ok(mut state_component) = state_query.single_mut() else {
        return;
    };

    // Collect node states from NodeSeverity components
    let mut node_states = HashMap::new();
    for node_severity in node_query.iter() {
        node_states.insert(node_severity.node_id.clone(), node_severity.severity);
    }

    // Construct input
    let input = PropagationInput { node_states };

    // Create emitter
    let mut emitter = BevyPropagationEmitter::new(&mut event_writer);

    // Call issun-core's pure propagation logic
    M::step(
        &graph.graph,
        &mut state_component.state,
        input,
        &mut emitter,
    );
}

/// System: Log propagation events for debugging.
///
/// This system listens to PropagationEventWrappers and logs them.
/// In a real game, you might use this to trigger VFX, SFX, or UI updates.
pub fn log_propagation_events(mut events: MessageReader<PropagationEventWrapper>) {
    for wrapper in events.read() {
        match &wrapper.event {
            PropagationEvent::PressureCalculated { node, pressure } => {
                info!("Node '{}' has infection pressure: {:.3}", node, pressure);
            }
            PropagationEvent::InitialInfection {
                node,
                initial_severity,
            } => {
                info!(
                    "Node '{}' triggered initial infection with severity: {}",
                    node, initial_severity
                );
            }
            PropagationEvent::PressureIncreased {
                node,
                old_pressure,
                new_pressure,
            } => {
                debug!(
                    "Node '{}' pressure increased: {:.3} -> {:.3}",
                    node, old_pressure, new_pressure
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use issun_core::mechanics::propagation::prelude::*;

    type TestPropagation = LinearPropagationMechanic;

    #[test]
    fn test_propagation_system_integration() {
        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins);

        // Create a simple graph: A -> B
        let graph = PropagationGraph::new(vec![PropagationEdge::new(
            "A".to_string(),
            "B".to_string(),
            0.5,
        )]);

        // Register resources and messages
        app.insert_resource(PropagationGraphResource::new(graph));
        app.add_message::<PropagationEventWrapper>();

        // Add system
        app.add_systems(Update, propagation_step_system::<TestPropagation>);

        // Spawn state component
        app.world_mut()
            .spawn(PropagationStateComponent::<TestPropagation>::new());

        // Spawn nodes: A infected, B clean
        app.world_mut()
            .spawn(NodeSeverity::new("A".to_string(), 100.0));
        app.world_mut()
            .spawn(NodeSeverity::new("B".to_string(), 0.0));

        // Run systems
        app.update();

        // Verify events were emitted
        let mut events = app
            .world_mut()
            .resource_mut::<Messages<PropagationEventWrapper>>();
        let event_list: Vec<_> = events.drain().collect();

        assert_eq!(event_list.len(), 2);

        // First event: PressureCalculated
        match &event_list[0].event {
            PropagationEvent::PressureCalculated { node, pressure } => {
                assert_eq!(node, "B");
                assert!((pressure - 0.5).abs() < 0.001);
            }
            _ => panic!("Expected PressureCalculated event"),
        }

        // Second event: InitialInfection
        match &event_list[1].event {
            PropagationEvent::InitialInfection {
                node,
                initial_severity,
            } => {
                assert_eq!(node, "B");
                assert_eq!(*initial_severity, 20);
            }
            _ => panic!("Expected InitialInfection event"),
        }
    }

    #[test]
    fn test_no_propagation_from_clean_nodes() {
        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins);

        let graph = PropagationGraph::new(vec![PropagationEdge::new(
            "A".to_string(),
            "B".to_string(),
            0.5,
        )]);

        app.insert_resource(PropagationGraphResource::new(graph));
        app.add_message::<PropagationEventWrapper>();
        app.add_systems(Update, propagation_step_system::<TestPropagation>);

        app.world_mut()
            .spawn(PropagationStateComponent::<TestPropagation>::new());

        // Both nodes clean
        app.world_mut()
            .spawn(NodeSeverity::new("A".to_string(), 0.0));
        app.world_mut()
            .spawn(NodeSeverity::new("B".to_string(), 0.0));

        app.update();

        // No events should be emitted
        let mut events = app
            .world_mut()
            .resource_mut::<Messages<PropagationEventWrapper>>();
        let event_list: Vec<_> = events.drain().collect();
        assert_eq!(event_list.len(), 0);
    }
}
