//! Propagation mechanic implementation

use crate::mechanics::{EventEmitter, Mechanic};
use std::marker::PhantomData;

use super::policies::PropagationPolicy;
use super::types::*;

/// Propagation mechanic - calculates infection spread across network topology
///
/// This mechanic implements graph-based disease propagation:
/// 1. For each node, calculate infection pressure from incoming edges
/// 2. If pressure exceeds threshold, trigger initial infection
/// 3. Emit events for pressure changes and infections
///
/// # Type Parameters
///
/// - `P`: PropagationPolicy that defines pressure calculation and infection thresholds
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::propagation::prelude::*;
/// use issun_core::mechanics::{Mechanic, EventEmitter};
/// use std::collections::HashMap;
///
/// // Event collector
/// struct Collector { events: Vec<PropagationEvent> }
/// impl EventEmitter<PropagationEvent> for Collector {
///     fn emit(&mut self, e: PropagationEvent) { self.events.push(e); }
/// }
///
/// // Create a simple graph: A -> B
/// let graph = PropagationGraph::new(vec![
///     PropagationEdge::new("A".to_string(), "B".to_string(), 0.5),
/// ]);
///
/// // A is infected (100), B is clean (0)
/// let mut node_states = HashMap::new();
/// node_states.insert("A".to_string(), 100.0);
/// node_states.insert("B".to_string(), 0.0);
///
/// let input = PropagationInput { node_states };
/// let mut state = PropagationState::default();
/// let mut emitter = Collector { events: Vec::new() };
///
/// // Run propagation
/// LinearPropagationMechanic::step(&graph, &mut state, input, &mut emitter);
///
/// // Check that B received pressure from A
/// assert!(state.get_pressure(&"B".to_string()) > 0.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PropagationMechanic<P: PropagationPolicy> {
    _marker: PhantomData<P>,
}

impl<P: PropagationPolicy> Mechanic for PropagationMechanic<P> {
    type Config = PropagationGraph;
    type State = PropagationState;
    type Input = PropagationInput;
    type Event = PropagationEvent;

    fn step(
        config: &Self::Config,
        state: &mut Self::State,
        input: Self::Input,
        emitter: &mut impl EventEmitter<Self::Event>,
    ) {
        // Clear previous pressures
        state.node_pressures.clear();

        // Get all unique target nodes
        let target_nodes: std::collections::HashSet<NodeId> = config
            .edges
            .iter()
            .map(|edge| edge.to.clone())
            .collect();

        // For each target node, calculate total infection pressure
        for target in target_nodes {
            let mut total_pressure = 0.0;

            // Sum pressure from all incoming edges
            for edge in config.incoming_edges(&target) {
                if let Some(&source_severity) = input.node_states.get(&edge.from) {
                    // Only calculate pressure if source is infected
                    if source_severity > 0.0 {
                        let pressure = P::calculate_pressure(source_severity, edge.rate);
                        total_pressure += pressure;
                    }
                }
            }

            // Store calculated pressure
            if total_pressure > 0.0 {
                state.node_pressures.insert(target.clone(), total_pressure);

                // Emit pressure calculated event
                emitter.emit(PropagationEvent::PressureCalculated {
                    node: target.clone(),
                    pressure: total_pressure,
                });

                // Check if we should trigger initial infection
                let target_severity = input.node_states.get(&target).copied().unwrap_or(0.0);

                if target_severity == 0.0 && P::should_trigger_infection(total_pressure) {
                    let initial_severity = P::calculate_initial_severity(total_pressure);
                    emitter.emit(PropagationEvent::InitialInfection {
                        node: target.clone(),
                        initial_severity,
                    });
                }
            }
        }
    }
}

/// Type alias for linear propagation mechanic
pub type LinearPropagationMechanic = PropagationMechanic<super::strategies::LinearPropagation>;

#[cfg(test)]
mod tests {
    use super::*;

    struct VecEmitter<E> {
        events: Vec<E>,
    }

    impl<E> EventEmitter<E> for VecEmitter<E> {
        fn emit(&mut self, event: E) {
            self.events.push(event);
        }
    }

    #[test]
    fn test_single_edge_propagation() {
        // A (infected) -> B (clean)
        let graph = PropagationGraph::new(vec![PropagationEdge::new(
            "A".to_string(),
            "B".to_string(),
            0.5,
        )]);

        let mut node_states = std::collections::HashMap::new();
        node_states.insert("A".to_string(), 100.0);
        node_states.insert("B".to_string(), 0.0);

        let input = PropagationInput { node_states };
        let mut state = PropagationState::default();
        let mut emitter = VecEmitter { events: Vec::new() };

        LinearPropagationMechanic::step(&graph, &mut state, input, &mut emitter);

        // B should have pressure from A
        let pressure_b = state.get_pressure(&"B".to_string());
        assert!((pressure_b - 0.5).abs() < 0.001); // 0.5 * (100/100)

        // Check events
        assert_eq!(emitter.events.len(), 2);
        assert!(matches!(
            emitter.events[0],
            PropagationEvent::PressureCalculated { ref node, pressure } if node == "B" && (pressure - 0.5).abs() < 0.001
        ));
        assert!(matches!(
            emitter.events[1],
            PropagationEvent::InitialInfection { ref node, initial_severity: 20 } if node == "B"
        ));
    }

    #[test]
    fn test_multiple_incoming_edges() {
        // A -> C, B -> C (both infected)
        let graph = PropagationGraph::new(vec![
            PropagationEdge::new("A".to_string(), "C".to_string(), 0.3),
            PropagationEdge::new("B".to_string(), "C".to_string(), 0.2),
        ]);

        let mut node_states = std::collections::HashMap::new();
        node_states.insert("A".to_string(), 100.0);
        node_states.insert("B".to_string(), 80.0);
        node_states.insert("C".to_string(), 0.0);

        let input = PropagationInput { node_states };
        let mut state = PropagationState::default();
        let mut emitter = VecEmitter { events: Vec::new() };

        LinearPropagationMechanic::step(&graph, &mut state, input, &mut emitter);

        // C should have accumulated pressure from both A and B
        let pressure_c = state.get_pressure(&"C".to_string());
        let expected = 0.3 * (100.0 / 100.0) + 0.2 * (80.0 / 100.0); // 0.3 + 0.16 = 0.46
        assert!((pressure_c - expected).abs() < 0.001);
    }

    #[test]
    fn test_no_propagation_from_clean_nodes() {
        // A (clean) -> B
        let graph = PropagationGraph::new(vec![PropagationEdge::new(
            "A".to_string(),
            "B".to_string(),
            0.5,
        )]);

        let mut node_states = std::collections::HashMap::new();
        node_states.insert("A".to_string(), 0.0);
        node_states.insert("B".to_string(), 0.0);

        let input = PropagationInput { node_states };
        let mut state = PropagationState::default();
        let mut emitter = VecEmitter { events: Vec::new() };

        LinearPropagationMechanic::step(&graph, &mut state, input, &mut emitter);

        // No pressure should be calculated
        assert_eq!(state.get_pressure(&"B".to_string()), 0.0);
        assert_eq!(emitter.events.len(), 0);
    }

    #[test]
    fn test_no_infection_below_threshold() {
        // A -> B with low edge rate
        let graph = PropagationGraph::new(vec![PropagationEdge::new(
            "A".to_string(),
            "B".to_string(),
            0.1,
        )]);

        let mut node_states = std::collections::HashMap::new();
        node_states.insert("A".to_string(), 100.0);
        node_states.insert("B".to_string(), 0.0);

        let input = PropagationInput { node_states };
        let mut state = PropagationState::default();
        let mut emitter = VecEmitter { events: Vec::new() };

        LinearPropagationMechanic::step(&graph, &mut state, input, &mut emitter);

        // Pressure calculated but no initial infection (0.1 < 0.15 threshold)
        assert_eq!(emitter.events.len(), 1);
        assert!(matches!(
            emitter.events[0],
            PropagationEvent::PressureCalculated { .. }
        ));
    }

    #[test]
    fn test_already_infected_node() {
        // A -> B, but B is already infected
        let graph = PropagationGraph::new(vec![PropagationEdge::new(
            "A".to_string(),
            "B".to_string(),
            0.5,
        )]);

        let mut node_states = std::collections::HashMap::new();
        node_states.insert("A".to_string(), 100.0);
        node_states.insert("B".to_string(), 50.0); // Already infected

        let input = PropagationInput { node_states };
        let mut state = PropagationState::default();
        let mut emitter = VecEmitter { events: Vec::new() };

        LinearPropagationMechanic::step(&graph, &mut state, input, &mut emitter);

        // Pressure calculated but no initial infection event
        assert_eq!(emitter.events.len(), 1);
        assert!(matches!(
            emitter.events[0],
            PropagationEvent::PressureCalculated { .. }
        ));
    }

    #[test]
    fn test_complex_topology() {
        // Diamond: A -> B, A -> C, B -> D, C -> D
        let graph = PropagationGraph::new(vec![
            PropagationEdge::new("A".to_string(), "B".to_string(), 0.4),
            PropagationEdge::new("A".to_string(), "C".to_string(), 0.3),
            PropagationEdge::new("B".to_string(), "D".to_string(), 0.2),
            PropagationEdge::new("C".to_string(), "D".to_string(), 0.2),
        ]);

        let mut node_states = std::collections::HashMap::new();
        node_states.insert("A".to_string(), 100.0);
        node_states.insert("B".to_string(), 0.0);
        node_states.insert("C".to_string(), 0.0);
        node_states.insert("D".to_string(), 0.0);

        let input = PropagationInput { node_states };
        let mut state = PropagationState::default();
        let mut emitter = VecEmitter { events: Vec::new() };

        LinearPropagationMechanic::step(&graph, &mut state, input, &mut emitter);

        // B and C should be infected from A
        assert!(state.get_pressure(&"B".to_string()) > 0.15);
        assert!(state.get_pressure(&"C".to_string()) > 0.15);

        // D should have pressure but from clean nodes (B and C), so no actual pressure yet
        // (This step only considers current input states, not newly infected nodes)
        assert_eq!(state.get_pressure(&"D".to_string()), 0.0);
    }
}
