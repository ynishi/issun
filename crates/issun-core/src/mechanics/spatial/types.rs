//! Core types for the spatial mechanic.
//!
//! This module defines the fundamental data structures for spatial representation,
//! topology, and occupancy management.

use std::collections::{HashMap, HashSet};

/// Unique identifier for a spatial node.
pub type NodeId = String;

/// Unique identifier for an entity in the spatial system.
pub type EntityId = String;

/// 3D position in space.
///
/// Can be used for both 2D (ignore `z`) and 3D spatial representations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Position {
    /// Create a new 2D position (z = 0.0).
    pub fn new_2d(x: f32, y: f32) -> Self {
        Self { x, y, z: 0.0 }
    }

    /// Create a new 3D position.
    pub fn new_3d(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Calculate Euclidean distance to another position.
    pub fn distance_to(&self, other: &Position) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// Calculate Manhattan distance to another position (ignoring z).
    pub fn manhattan_distance_to(&self, other: &Position) -> f32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::new_2d(0.0, 0.0)
    }
}

/// Type of spatial node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeType {
    /// City or settlement
    City,
    /// Dungeon room
    Room,
    /// Grid cell
    Cell,
    /// Wilderness area
    Wilderness,
    /// Custom type (extensible)
    Custom,
}

/// A node in the spatial graph.
#[derive(Debug, Clone)]
pub struct SpatialNode {
    /// Unique identifier
    pub id: NodeId,

    /// Optional position (for coordinate-based topologies)
    pub position: Option<Position>,

    /// Maximum occupancy (0 = unlimited)
    pub capacity: usize,

    /// Type of node
    pub node_type: NodeType,
}

impl SpatialNode {
    /// Create a new spatial node.
    pub fn new(id: impl Into<String>, node_type: NodeType) -> Self {
        Self {
            id: id.into(),
            position: None,
            capacity: 0, // Unlimited by default
            node_type,
        }
    }

    /// Set position.
    pub fn with_position(mut self, position: Position) -> Self {
        self.position = Some(position);
        self
    }

    /// Set capacity.
    pub fn with_capacity(mut self, capacity: usize) -> Self {
        self.capacity = capacity;
        self
    }

    /// Check if node has a position.
    pub fn has_position(&self) -> bool {
        self.position.is_some()
    }

    /// Check if node has unlimited capacity.
    pub fn is_unlimited(&self) -> bool {
        self.capacity == 0
    }
}

/// An edge connecting two nodes in the spatial graph.
#[derive(Debug, Clone)]
pub struct SpatialEdge {
    /// Source node
    pub from: NodeId,

    /// Target node
    pub to: NodeId,

    /// Movement cost (distance, travel time, etc.)
    pub cost: f32,

    /// Whether the edge can be traversed in both directions
    pub bidirectional: bool,
}

impl SpatialEdge {
    /// Create a new unidirectional edge.
    pub fn new(from: impl Into<String>, to: impl Into<String>, cost: f32) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            cost,
            bidirectional: false,
        }
    }

    /// Create a new bidirectional edge.
    pub fn new_bidirectional(from: impl Into<String>, to: impl Into<String>, cost: f32) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            cost,
            bidirectional: true,
        }
    }

    /// Get the reverse edge (if bidirectional).
    pub fn reverse(&self) -> Option<Self> {
        if self.bidirectional {
            Some(Self {
                from: self.to.clone(),
                to: self.from.clone(),
                cost: self.cost,
                bidirectional: true,
            })
        } else {
            None
        }
    }
}

/// Metadata for spatial graph configuration.
#[derive(Debug, Clone, Default)]
pub struct GraphMetadata {
    /// Grid dimensions (if using grid topology)
    pub grid_dimensions: Option<(usize, usize)>,

    /// Whether to use diagonal connections in grid
    pub grid_diagonal: bool,

    /// Custom metadata (extensible)
    pub custom: HashMap<String, String>,
}

/// The spatial graph configuration.
///
/// Represents the static structure of the spatial world.
#[derive(Debug, Clone)]
pub struct SpatialGraph {
    /// Nodes in the graph
    pub nodes: HashMap<NodeId, SpatialNode>,

    /// Edges connecting nodes
    pub edges: Vec<SpatialEdge>,

    /// Graph metadata
    pub metadata: GraphMetadata,
}

impl SpatialGraph {
    /// Create a new empty spatial graph.
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            metadata: GraphMetadata::default(),
        }
    }

    /// Create a graph from nodes and edges.
    pub fn from_parts(
        nodes: Vec<SpatialNode>,
        edges: Vec<SpatialEdge>,
        metadata: GraphMetadata,
    ) -> Self {
        let nodes = nodes.into_iter().map(|n| (n.id.clone(), n)).collect();
        Self {
            nodes,
            edges,
            metadata,
        }
    }

    /// Add a node to the graph.
    pub fn add_node(&mut self, node: SpatialNode) {
        self.nodes.insert(node.id.clone(), node);
    }

    /// Add an edge to the graph.
    pub fn add_edge(&mut self, edge: SpatialEdge) {
        self.edges.push(edge);
    }

    /// Get all edges starting from a node.
    pub fn outgoing_edges<'a>(&'a self, node: &'a NodeId) -> impl Iterator<Item = &'a SpatialEdge> + 'a {
        self.edges.iter().filter(move |e| &e.from == node)
    }

    /// Get all edges ending at a node.
    pub fn incoming_edges<'a>(&'a self, node: &'a NodeId) -> impl Iterator<Item = &'a SpatialEdge> + 'a {
        self.edges.iter().filter(move |e| &e.to == node)
    }

    /// Check if a node exists.
    pub fn has_node(&self, node: &NodeId) -> bool {
        self.nodes.contains_key(node)
    }

    /// Get a node by ID.
    pub fn get_node(&self, node: &NodeId) -> Option<&SpatialNode> {
        self.nodes.get(node)
    }
}

impl Default for SpatialGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Occupancy state tracking which entities are at which locations.
#[derive(Debug, Clone, Default)]
pub struct OccupancyState {
    /// Map of node -> set of entities occupying it
    pub occupants: HashMap<NodeId, HashSet<EntityId>>,
}

impl OccupancyState {
    /// Create a new empty occupancy state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an entity to a node.
    pub fn add_occupant(&mut self, node: NodeId, entity: EntityId) {
        self.occupants.entry(node).or_default().insert(entity);
    }

    /// Remove an entity from a node.
    pub fn remove_occupant(&mut self, node: &NodeId, entity: &EntityId) -> bool {
        if let Some(occupants) = self.occupants.get_mut(node) {
            occupants.remove(entity)
        } else {
            false
        }
    }

    /// Move an entity from one node to another.
    pub fn move_occupant(&mut self, entity: EntityId, from: &NodeId, to: NodeId) -> bool {
        if self.remove_occupant(from, &entity) {
            self.add_occupant(to, entity);
            true
        } else {
            false
        }
    }

    /// Get all entities at a node.
    pub fn get_occupants(&self, node: &NodeId) -> Option<&HashSet<EntityId>> {
        self.occupants.get(node)
    }

    /// Get number of entities at a node.
    pub fn count_occupants(&self, node: &NodeId) -> usize {
        self.occupants.get(node).map_or(0, |s| s.len())
    }

    /// Check if a node has room for more entities.
    pub fn has_capacity(&self, graph: &SpatialGraph, node: &NodeId) -> bool {
        if let Some(spatial_node) = graph.get_node(node) {
            if spatial_node.is_unlimited() {
                return true;
            }
            self.count_occupants(node) < spatial_node.capacity
        } else {
            false
        }
    }
}

/// Input query for the spatial mechanic.
#[derive(Debug, Clone)]
pub enum SpatialQuery {
    /// Get all neighbors of a node
    Neighbors { node: NodeId },

    /// Calculate distance between two nodes
    Distance { from: NodeId, to: NodeId },

    /// Check if movement is allowed
    CanMove { from: NodeId, to: NodeId },

    /// Get all occupants at a location
    GetOccupants { node: NodeId },

    /// Update occupancy (entity moved)
    UpdateOccupancy {
        entity: EntityId,
        from: Option<NodeId>,
        to: NodeId,
    },
}

/// Reason why movement is blocked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockReason {
    /// No connection exists between nodes
    NoConnection,
    /// Target node is at capacity
    CapacityFull,
    /// Edge is impassable
    Impassable,
    /// Source node doesn't exist
    InvalidSource,
    /// Target node doesn't exist
    InvalidTarget,
}

/// Events emitted by the spatial mechanic.
#[derive(Debug, Clone)]
pub enum SpatialEvent {
    /// Neighbors query result
    NeighborsFound {
        node: NodeId,
        neighbors: Vec<NodeId>,
    },

    /// Distance calculation result
    DistanceCalculated {
        from: NodeId,
        to: NodeId,
        distance: f32,
    },

    /// Movement is allowed
    MovementAllowed {
        from: NodeId,
        to: NodeId,
        cost: f32,
    },

    /// Movement is blocked
    MovementBlocked {
        from: NodeId,
        to: NodeId,
        reason: BlockReason,
    },

    /// Occupancy changed (entity entered/left)
    OccupancyChanged {
        node: NodeId,
        entity: EntityId,
        entered: bool,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_2d() {
        let p1 = Position::new_2d(0.0, 0.0);
        let p2 = Position::new_2d(3.0, 4.0);

        assert_eq!(p1.distance_to(&p2), 5.0);
        assert_eq!(p1.manhattan_distance_to(&p2), 7.0);
    }

    #[test]
    fn test_spatial_node() {
        let node = SpatialNode::new("city1", NodeType::City)
            .with_position(Position::new_2d(10.0, 20.0))
            .with_capacity(100);

        assert_eq!(node.id, "city1");
        assert_eq!(node.node_type, NodeType::City);
        assert!(node.has_position());
        assert!(!node.is_unlimited());
        assert_eq!(node.capacity, 100);
    }

    #[test]
    fn test_spatial_edge_bidirectional() {
        let edge = SpatialEdge::new_bidirectional("A", "B", 10.0);
        assert!(edge.bidirectional);

        let reverse = edge.reverse().unwrap();
        assert_eq!(reverse.from, "B");
        assert_eq!(reverse.to, "A");
        assert_eq!(reverse.cost, 10.0);
    }

    #[test]
    fn test_spatial_graph() {
        let mut graph = SpatialGraph::new();

        graph.add_node(SpatialNode::new("A", NodeType::City));
        graph.add_node(SpatialNode::new("B", NodeType::City));
        graph.add_edge(SpatialEdge::new("A", "B", 5.0));

        assert!(graph.has_node(&"A".to_string()));
        assert!(graph.has_node(&"B".to_string()));
        assert_eq!(graph.outgoing_edges(&"A".to_string()).count(), 1);
    }

    #[test]
    fn test_occupancy_state() {
        let mut state = OccupancyState::new();
        let mut graph = SpatialGraph::new();
        graph.add_node(SpatialNode::new("room1", NodeType::Room).with_capacity(2));

        state.add_occupant("room1".to_string(), "entity1".to_string());
        assert_eq!(state.count_occupants(&"room1".to_string()), 1);
        assert!(state.has_capacity(&graph, &"room1".to_string()));

        state.add_occupant("room1".to_string(), "entity2".to_string());
        assert_eq!(state.count_occupants(&"room1".to_string()), 2);
        assert!(!state.has_capacity(&graph, &"room1".to_string()));

        let moved = state.move_occupant(
            "entity1".to_string(),
            &"room1".to_string(),
            "room2".to_string(),
        );
        assert!(moved);
        assert_eq!(state.count_occupants(&"room1".to_string()), 1);
        assert_eq!(state.count_occupants(&"room2".to_string()), 1);
    }
}
