//! The core SpatialMechanic implementation.

use std::marker::PhantomData;

use crate::mechanics::{EventEmitter, Mechanic, Transactional};

use super::policies::{DistancePolicy, TopologyPolicy};
use super::strategies::{FixedDistance, GraphTopology};
use super::types::{
    BlockReason, OccupancyState, SpatialEvent, SpatialGraph, SpatialQuery,
};

/// The core spatial mechanic that composes topology and distance policies.
///
/// # Type Parameters
///
/// - `T`: Topology policy (defines adjacency and neighbors)
/// - `D`: Distance policy (calculates distances and costs)
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::spatial::prelude::*;
/// use issun_core::mechanics::{Mechanic, EventEmitter};
///
/// // Use default policies (GraphTopology + FixedDistance)
/// type SimpleSpatial = SpatialMechanic;
///
/// // Or customize
/// type GridSpatial = SpatialMechanic<GridTopology, ManhattanDistance>;
///
/// // Create graph and state
/// let mut graph = SpatialGraph::new();
/// graph.add_node(SpatialNode::new("A", NodeType::City));
/// graph.add_node(SpatialNode::new("B", NodeType::City));
/// graph.add_edge(SpatialEdge::new("A", "B", 10.0));
///
/// let mut state = OccupancyState::new();
///
/// // Query neighbors
/// let query = SpatialQuery::Neighbors { node: "A".to_string() };
///
/// // Event collector
/// # struct VecEmitter(Vec<SpatialEvent>);
/// # impl EventEmitter<SpatialEvent> for VecEmitter {
/// #     fn emit(&mut self, event: SpatialEvent) { self.0.push(event); }
/// # }
/// let mut emitter = VecEmitter(vec![]);
///
/// // Execute query
/// SimpleSpatial::step(&graph, &mut state, query, &mut emitter);
/// ```
pub struct SpatialMechanic<T: TopologyPolicy = GraphTopology, D: DistancePolicy = FixedDistance> {
    _marker: PhantomData<(T, D)>,
}

impl<T, D> Mechanic for SpatialMechanic<T, D>
where
    T: TopologyPolicy + Default,
    D: DistancePolicy + Default,
{
    type Config = SpatialGraph;
    type State = OccupancyState;
    type Input = SpatialQuery;
    type Event = SpatialEvent;

    // Spatial queries require consistent snapshot of world state
    type Execution = Transactional;

    fn step(
        config: &Self::Config,
        state: &mut Self::State,
        input: Self::Input,
        emitter: &mut impl EventEmitter<Self::Event>,
    ) {
        let topology = T::default();
        let distance = D::default();

        match input {
            SpatialQuery::Neighbors { node } => {
                let neighbors = topology.neighbors(config, &node);
                emitter.emit(SpatialEvent::NeighborsFound { node, neighbors });
            }

            SpatialQuery::Distance { from, to } => {
                if let Some(dist) = distance.calculate_distance(config, &from, &to) {
                    emitter.emit(SpatialEvent::DistanceCalculated {
                        from,
                        to,
                        distance: dist,
                    });
                } else {
                    // No distance available (nodes don't exist or not connected)
                    emitter.emit(SpatialEvent::MovementBlocked {
                        from,
                        to,
                        reason: BlockReason::NoConnection,
                    });
                }
            }

            SpatialQuery::CanMove { from, to } => {
                // Validate nodes exist
                if !config.has_node(&from) {
                    emitter.emit(SpatialEvent::MovementBlocked {
                        from,
                        to,
                        reason: BlockReason::InvalidSource,
                    });
                    return;
                }

                if !config.has_node(&to) {
                    emitter.emit(SpatialEvent::MovementBlocked {
                        from,
                        to,
                        reason: BlockReason::InvalidTarget,
                    });
                    return;
                }

                // Check topology
                if !topology.can_move(config, &from, &to) {
                    emitter.emit(SpatialEvent::MovementBlocked {
                        from,
                        to,
                        reason: BlockReason::NoConnection,
                    });
                    return;
                }

                // Check capacity
                if !state.has_capacity(config, &to) {
                    emitter.emit(SpatialEvent::MovementBlocked {
                        from,
                        to,
                        reason: BlockReason::CapacityFull,
                    });
                    return;
                }

                // Movement allowed, emit with cost
                let cost = distance.movement_cost(config, &from, &to).unwrap_or(1.0);
                emitter.emit(SpatialEvent::MovementAllowed { from, to, cost });
            }

            SpatialQuery::GetOccupants { node } => {
                // This query doesn't emit an event, just for state inspection
                // Could add OccupantsRetrieved event if needed
                let _occupants = state.get_occupants(&node);
            }

            SpatialQuery::UpdateOccupancy { entity, from, to } => {
                if let Some(from_node) = from {
                    // Move entity from one node to another
                    if state.move_occupant(entity.clone(), &from_node, to.clone()) {
                        emitter.emit(SpatialEvent::OccupancyChanged {
                            node: from_node,
                            entity: entity.clone(),
                            entered: false,
                        });
                        emitter.emit(SpatialEvent::OccupancyChanged {
                            node: to,
                            entity,
                            entered: true,
                        });
                    }
                } else {
                    // Add entity to node (spawn)
                    state.add_occupant(to.clone(), entity.clone());
                    emitter.emit(SpatialEvent::OccupancyChanged {
                        node: to,
                        entity,
                        entered: true,
                    });
                }
            }
        }
    }
}

/// Type alias for simple graph-based spatial mechanic.
pub type GraphSpatialMechanic = SpatialMechanic<GraphTopology, FixedDistance>;

/// Type alias for grid-based spatial mechanic with Manhattan distance.
pub type GridSpatialMechanic =
    SpatialMechanic<super::strategies::GridTopology, super::strategies::ManhattanDistance>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mechanics::spatial::types::{NodeType, SpatialEdge, SpatialNode};

    struct VecEmitter {
        events: Vec<SpatialEvent>,
    }

    impl EventEmitter<SpatialEvent> for VecEmitter {
        fn emit(&mut self, event: SpatialEvent) {
            self.events.push(event);
        }
    }

    #[test]
    fn test_neighbors_query() {
        let mut graph = SpatialGraph::new();
        graph.add_node(SpatialNode::new("A", NodeType::City));
        graph.add_node(SpatialNode::new("B", NodeType::City));
        graph.add_edge(SpatialEdge::new("A", "B", 1.0));

        let mut state = OccupancyState::new();
        let query = SpatialQuery::Neighbors {
            node: "A".to_string(),
        };
        let mut emitter = VecEmitter { events: Vec::new() };

        GraphSpatialMechanic::step(&graph, &mut state, query, &mut emitter);

        assert_eq!(emitter.events.len(), 1);
        match &emitter.events[0] {
            SpatialEvent::NeighborsFound { node, neighbors } => {
                assert_eq!(node, "A");
                assert_eq!(neighbors, &vec!["B".to_string()]);
            }
            _ => panic!("Expected NeighborsFound event"),
        }
    }

    #[test]
    fn test_distance_query() {
        let mut graph = SpatialGraph::new();
        graph.add_node(SpatialNode::new("A", NodeType::City));
        graph.add_node(SpatialNode::new("B", NodeType::City));
        graph.add_edge(SpatialEdge::new("A", "B", 10.0));

        let mut state = OccupancyState::new();
        let query = SpatialQuery::Distance {
            from: "A".to_string(),
            to: "B".to_string(),
        };
        let mut emitter = VecEmitter { events: Vec::new() };

        GraphSpatialMechanic::step(&graph, &mut state, query, &mut emitter);

        assert_eq!(emitter.events.len(), 1);
        match &emitter.events[0] {
            SpatialEvent::DistanceCalculated { from, to, distance } => {
                assert_eq!(from, "A");
                assert_eq!(to, "B");
                assert_eq!(*distance, 1.0); // FixedDistance always returns 1.0
            }
            _ => panic!("Expected DistanceCalculated event"),
        }
    }

    #[test]
    fn test_can_move_allowed() {
        let mut graph = SpatialGraph::new();
        graph.add_node(SpatialNode::new("A", NodeType::Room).with_capacity(10));
        graph.add_node(SpatialNode::new("B", NodeType::Room).with_capacity(10));
        graph.add_edge(SpatialEdge::new("A", "B", 1.0));

        let mut state = OccupancyState::new();
        let query = SpatialQuery::CanMove {
            from: "A".to_string(),
            to: "B".to_string(),
        };
        let mut emitter = VecEmitter { events: Vec::new() };

        GraphSpatialMechanic::step(&graph, &mut state, query, &mut emitter);

        assert_eq!(emitter.events.len(), 1);
        match &emitter.events[0] {
            SpatialEvent::MovementAllowed { from, to, cost } => {
                assert_eq!(from, "A");
                assert_eq!(to, "B");
                assert_eq!(*cost, 1.0);
            }
            _ => panic!("Expected MovementAllowed event"),
        }
    }

    #[test]
    fn test_can_move_capacity_full() {
        let mut graph = SpatialGraph::new();
        graph.add_node(SpatialNode::new("A", NodeType::Room).with_capacity(10));
        graph.add_node(SpatialNode::new("B", NodeType::Room).with_capacity(1));
        graph.add_edge(SpatialEdge::new("A", "B", 1.0));

        let mut state = OccupancyState::new();
        state.add_occupant("B".to_string(), "entity1".to_string());

        let query = SpatialQuery::CanMove {
            from: "A".to_string(),
            to: "B".to_string(),
        };
        let mut emitter = VecEmitter { events: Vec::new() };

        GraphSpatialMechanic::step(&graph, &mut state, query, &mut emitter);

        assert_eq!(emitter.events.len(), 1);
        match &emitter.events[0] {
            SpatialEvent::MovementBlocked { reason, .. } => {
                assert_eq!(*reason, BlockReason::CapacityFull);
            }
            _ => panic!("Expected MovementBlocked event"),
        }
    }

    #[test]
    fn test_update_occupancy() {
        let mut graph = SpatialGraph::new();
        graph.add_node(SpatialNode::new("A", NodeType::City));
        graph.add_node(SpatialNode::new("B", NodeType::City));

        let mut state = OccupancyState::new();
        state.add_occupant("A".to_string(), "entity1".to_string());

        let query = SpatialQuery::UpdateOccupancy {
            entity: "entity1".to_string(),
            from: Some("A".to_string()),
            to: "B".to_string(),
        };
        let mut emitter = VecEmitter { events: Vec::new() };

        GraphSpatialMechanic::step(&graph, &mut state, query, &mut emitter);

        assert_eq!(emitter.events.len(), 2);
        assert_eq!(state.count_occupants(&"A".to_string()), 0);
        assert_eq!(state.count_occupants(&"B".to_string()), 1);
    }
}
