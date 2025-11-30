//! Spatial mechanic: Unified space and topology system.
//!
//! This module provides a policy-based spatial representation system that
//! consolidates spatial logic across territory, worldmap, and dungeon systems.
//!
//! # Architecture
//!
//! The spatial mechanic follows **Policy-Based Design**:
//! - The core `SpatialMechanic<T, D>` is generic over two policies
//! - `T: TopologyPolicy` determines how adjacency and neighbors are calculated
//! - `D: DistancePolicy` determines how distances are measured
//! - All logic is resolved at compile time via static dispatch
//!
//! # Quick Start
//!
//! ```
//! use issun_core::mechanics::spatial::prelude::*;
//! use issun_core::mechanics::{Mechanic, EventEmitter};
//!
//! // Create a simple graph
//! let mut graph = SpatialGraph::new();
//! graph.add_node(SpatialNode::new("City_A", NodeType::City));
//! graph.add_node(SpatialNode::new("City_B", NodeType::City));
//! graph.add_edge(SpatialEdge::new("City_A", "City_B", 100.0));
//!
//! // Query neighbors
//! let mut state = OccupancyState::new();
//! let query = SpatialQuery::Neighbors { node: "City_A".to_string() };
//!
//! # struct TestEmitter { events: Vec<SpatialEvent> }
//! # impl EventEmitter<SpatialEvent> for TestEmitter {
//! #     fn emit(&mut self, event: SpatialEvent) { self.events.push(event); }
//! # }
//! let mut emitter = TestEmitter { events: vec![] };
//!
//! // Execute spatial mechanic
//! GraphSpatialMechanic::step(&graph, &mut state, query, &mut emitter);
//! ```
//!
//! # Module Organization
//!
//! - `types`: Core data structures (SpatialGraph, SpatialNode, Position, etc.)
//! - `policies`: Policy traits (TopologyPolicy, DistancePolicy)
//! - `strategies`: Concrete implementations of policies
//! - `mechanic`: The core `SpatialMechanic<T, D>` implementation
//! - `prelude`: Convenient re-exports for common use
//!
//! # Policy Combinations
//!
//! ## Topology Policies
//! - `GraphTopology`: Uses explicit edges (most flexible)
//! - `GridTopology`: 2D grid with 4-way or 8-way connectivity
//!
//! ## Distance Policies
//! - `FixedDistance`: All edges = 1.0 distance (hop count)
//! - `EuclideanDistance`: Straight-line distance using positions
//! - `ManhattanDistance`: Grid-based distance (no diagonals)
//!
//! # Examples
//!
//! ## Example 1: Graph-Based World Map
//!
//! ```
//! use issun_core::mechanics::spatial::prelude::*;
//!
//! let mut world = SpatialGraph::new();
//! world.add_node(
//!     SpatialNode::new("London", NodeType::City)
//!         .with_position(Position::new_2d(0.0, 51.5))
//! );
//! world.add_node(
//!     SpatialNode::new("Paris", NodeType::City)
//!         .with_position(Position::new_2d(2.3, 48.9))
//! );
//! world.add_edge(SpatialEdge::new_bidirectional("London", "Paris", 344.0)); // km
//! ```
//!
//! ## Example 2: Grid-Based Dungeon
//!
//! ```
//! use issun_core::mechanics::spatial::prelude::*;
//!
//! let mut dungeon = SpatialGraph::new();
//! for y in 0..10 {
//!     for x in 0..10 {
//!         let id = format!("{},{}", x, y);
//!         dungeon.add_node(
//!             SpatialNode::new(&id, NodeType::Cell)
//!                 .with_position(Position::new_2d(x as f32, y as f32))
//!         );
//!     }
//! }
//!
//! // Use GridTopology + ManhattanDistance for grid movement
//! type DungeonSpatial = SpatialMechanic<GridTopology, ManhattanDistance>;
//! ```

pub mod conversions;
pub mod mechanic;
pub mod policies;
pub mod strategies;
pub mod types;

// Re-export main types
pub use mechanic::{GraphSpatialMechanic, GridSpatialMechanic, SpatialMechanic};
pub use policies::{DistancePolicy, TopologyPolicy};
pub use types::{
    BlockReason, EntityId, GraphMetadata, NodeId, NodeType, OccupancyState, Position, SpatialEdge,
    SpatialEvent, SpatialGraph, SpatialNode, SpatialQuery,
};

/// Prelude module for convenient imports.
///
/// Import everything needed to use the spatial mechanic:
///
/// ```
/// use issun_core::mechanics::spatial::prelude::*;
/// ```
pub mod prelude {
    pub use super::mechanic::{GraphSpatialMechanic, GridSpatialMechanic, SpatialMechanic};
    pub use super::policies::{DistancePolicy, TopologyPolicy};
    pub use super::strategies::{
        EuclideanDistance, FixedDistance, GraphTopology, GridTopology, ManhattanDistance,
    };
    pub use super::types::{
        BlockReason, EntityId, GraphMetadata, NodeId, NodeType, OccupancyState, Position,
        SpatialEdge, SpatialEvent, SpatialGraph, SpatialNode, SpatialQuery,
    };
}
