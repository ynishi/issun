//! PropagationPluginV2: Policy-Based Propagation System for Bevy.
//!
//! This plugin integrates issun-core's policy-based propagation mechanic with Bevy's ECS.

use bevy::prelude::*;
use issun_core::mechanics::propagation::{PropagationEvent, PropagationGraph, PropagationInput};
use issun_core::mechanics::Mechanic;
use std::marker::PhantomData;

use super::systems::{log_propagation_events, propagation_step_system};
use super::types::{
    NodeSeverity, PropagationEventWrapper, PropagationGraphResource, PropagationStateComponent,
};

/// Propagation plugin using issun-core's policy-based design.
///
/// This plugin is generic over the propagation mechanic type, allowing you to
/// choose different propagation behaviors at compile time.
///
/// # Type Parameters
///
/// - `M`: The propagation mechanic to use (must implement `Mechanic` with appropriate types)
///
/// # Examples
///
/// ```ignore
/// use bevy::prelude::*;
/// use issun_bevy::plugins::propagation_v2::PropagationPluginV2;
/// use issun_core::mechanics::propagation::prelude::*;
///
/// // Linear propagation
/// App::new()
///     .add_plugins(PropagationPluginV2::<LinearPropagationMechanic>::new(graph))
///     .run();
/// ```
pub struct PropagationPluginV2<M>
where
    M: Mechanic<
        Config = PropagationGraph,
        State = issun_core::mechanics::propagation::PropagationState,
        Input = PropagationInput,
        Event = PropagationEvent,
    >,
{
    /// Propagation graph configuration
    pub graph: PropagationGraph,

    /// Phantom data to hold the mechanic type
    _phantom: PhantomData<M>,
}

impl<M> PropagationPluginV2<M>
where
    M: Mechanic<
        Config = PropagationGraph,
        State = issun_core::mechanics::propagation::PropagationState,
        Input = PropagationInput,
        Event = PropagationEvent,
    >,
{
    /// Create a new propagation plugin with the given graph.
    pub fn new(graph: PropagationGraph) -> Self {
        Self {
            graph,
            _phantom: PhantomData,
        }
    }
}

impl<M> Plugin for PropagationPluginV2<M>
where
    M: Mechanic<
            Config = PropagationGraph,
            State = issun_core::mechanics::propagation::PropagationState,
            Input = PropagationInput,
            Event = PropagationEvent,
        > + Send
        + Sync
        + 'static,
{
    fn build(&self, app: &mut App) {
        // Register resources - wrap issun-core's graph
        app.insert_resource(PropagationGraphResource::new(self.graph.clone()));

        // Register component types
        app.register_type::<NodeSeverity>();

        // Register messages - use wrapper for issun-core events
        app.add_message::<PropagationEventWrapper>();

        // Spawn a single PropagationStateComponent for this mechanic
        app.world_mut().spawn(PropagationStateComponent::<M>::new());

        // Register systems
        app.add_systems(
            Update,
            (propagation_step_system::<M>, log_propagation_events),
        );

        info!(
            "PropagationPluginV2 initialized with mechanic: {}",
            std::any::type_name::<M>()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use issun_core::mechanics::propagation::prelude::*;

    type TestPropagation = LinearPropagationMechanic;

    #[test]
    fn test_plugin_builds() {
        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins);

        let graph = PropagationGraph::new(vec![PropagationEdge::new(
            "A".to_string(),
            "B".to_string(),
            0.5,
        )]);

        app.add_plugins(PropagationPluginV2::<TestPropagation>::new(graph));

        // Verify resource exists
        assert!(app.world().contains_resource::<PropagationGraphResource>());

        // Verify state component was spawned
        let state_count = app
            .world_mut()
            .query::<&PropagationStateComponent<TestPropagation>>()
            .iter(app.world())
            .count();
        assert_eq!(state_count, 1);
    }

    #[test]
    fn test_full_propagation_flow() {
        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins);

        // Create graph: A -> B
        let graph = PropagationGraph::new(vec![PropagationEdge::new(
            "A".to_string(),
            "B".to_string(),
            0.5,
        )]);

        app.add_plugins(PropagationPluginV2::<TestPropagation>::new(graph));

        // Spawn nodes
        app.world_mut()
            .spawn(NodeSeverity::new("A".to_string(), 100.0));
        app.world_mut()
            .spawn(NodeSeverity::new("B".to_string(), 0.0));

        // Run one update
        app.update();

        // Verify events were published
        let mut events = app
            .world_mut()
            .resource_mut::<Messages<PropagationEventWrapper>>();
        let event_list: Vec<_> = events.drain().collect();

        assert_eq!(event_list.len(), 2);

        // Check PressureCalculated
        match &event_list[0].event {
            PropagationEvent::PressureCalculated { node, pressure } => {
                assert_eq!(node, "B");
                assert!((pressure - 0.5).abs() < 0.001);
            }
            _ => panic!("Expected PressureCalculated"),
        }

        // Check InitialInfection
        match &event_list[1].event {
            PropagationEvent::InitialInfection {
                node,
                initial_severity,
            } => {
                assert_eq!(node, "B");
                assert_eq!(*initial_severity, 20);
            }
            _ => panic!("Expected InitialInfection"),
        }
    }

    #[test]
    fn test_complex_graph() {
        let mut app = App::new();
        app.add_plugins(bevy::MinimalPlugins);

        // Diamond graph: A -> B, A -> C, B -> D, C -> D
        let graph = PropagationGraph::new(vec![
            PropagationEdge::new("A".to_string(), "B".to_string(), 0.4),
            PropagationEdge::new("A".to_string(), "C".to_string(), 0.3),
            PropagationEdge::new("B".to_string(), "D".to_string(), 0.2),
            PropagationEdge::new("C".to_string(), "D".to_string(), 0.2),
        ]);

        app.add_plugins(PropagationPluginV2::<TestPropagation>::new(graph));

        // Spawn nodes: A infected, others clean
        app.world_mut()
            .spawn(NodeSeverity::new("A".to_string(), 100.0));
        app.world_mut()
            .spawn(NodeSeverity::new("B".to_string(), 0.0));
        app.world_mut()
            .spawn(NodeSeverity::new("C".to_string(), 0.0));
        app.world_mut()
            .spawn(NodeSeverity::new("D".to_string(), 0.0));

        app.update();

        // Verify B and C received pressure and infection
        let mut events = app
            .world_mut()
            .resource_mut::<Messages<PropagationEventWrapper>>();
        let event_list: Vec<_> = events.drain().collect();

        // Should have 4 events: 2x PressureCalculated + 2x InitialInfection for B and C
        assert_eq!(event_list.len(), 4);
    }
}
